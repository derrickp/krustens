use std::{io, sync::Arc, time::Duration};

use crossterm::{
    event::{self, poll, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::sync::Mutex;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::{
    app::{Application, Mode},
    errors::InteractiveError,
    persistence::EventStore,
    projections::ListenTrackerRepository,
};

use unicode_width::UnicodeWidthStr;

pub async fn full_ui(
    store: Arc<dyn EventStore>,
    repository: Arc<Mutex<dyn ListenTrackerRepository>>,
) -> Result<(), InteractiveError> {
    let mut app = Application::new(store, repository);
    app.initialize().await?;

    println!("Loading...");

    enable_raw_mode().map_err(|e| InteractiveError::Crossterm {
        message: e.to_string(),
    })?;

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture).map_err(|e| {
        InteractiveError::Crossterm {
            message: e.to_string(),
        }
    })?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| InteractiveError::TuiError {
        message: e.to_string(),
    })?;

    // create app and run it
    let res = run_app(&mut terminal, app).await;

    // restore terminal
    disable_raw_mode().map_err(|e| InteractiveError::Crossterm {
        message: e.to_string(),
    })?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .map_err(|e| InteractiveError::Crossterm {
        message: e.to_string(),
    })?;
    terminal
        .show_cursor()
        .map_err(|e| InteractiveError::TuiError {
            message: e.to_string(),
        })?;

    if let Err(err) = res {
        println!("{err:?}")
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: Application) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Poll for an event, we do this so that we don't block on waiting for an event.
        // If we blocked, then we wouldn't tick the app to the next stage.
        if poll(Duration::from_millis(15))? {
            if let Event::Key(key) = event::read()? {
                match app.state.mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            app.start_command_input();
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Char('c') => {
                            app.copy_to_clipboard();
                        }
                        KeyCode::Right => {
                            app.go_to_next_page();
                        }
                        KeyCode::Left => {
                            app.go_to_previous_page();
                        }
                        _ => {}
                    },
                    Mode::EnterCommand => match key.code {
                        KeyCode::Enter => {
                            app.command_name_entered();
                        }
                        KeyCode::Char(c) => {
                            app.push_input_char(c);
                        }
                        KeyCode::Backspace => {
                            app.pop_input_char();
                        }
                        KeyCode::Esc => {
                            app.cancel_command();
                        }
                        KeyCode::Tab => {
                            app.autocomplete_command_name();
                        }
                        _ => {}
                    },
                    Mode::CommandParameters => match key.code {
                        KeyCode::Enter => {
                            app.advance_command_input();
                        }
                        KeyCode::Char(c) => {
                            app.push_input_char(c);
                        }
                        KeyCode::Backspace => {
                            app.pop_input_char();
                        }
                        KeyCode::Esc => {
                            app.cancel_command();
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        } else {
            // Timeout expired, no `Event` is available
        }

        app.tick().await.unwrap();
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &Application) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.mode() {
        Mode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to enter command, "),
                Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to copy all output, "),
                Span::styled("< and >", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to go back and forward in output."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        Mode::EnterCommand => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to execute the command"),
            ],
            Style::default(),
        ),
        Mode::CommandParameters => {
            if let Some(description) = app.current_parameter_description() {
                (vec![Span::raw(description)], Style::default())
            } else {
                (Vec::new(), Style::default())
            }
        }
        _ => (Vec::new(), Style::default()),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.current_input())
        .style(match app.mode() {
            Mode::EnterCommand | Mode::CommandParameters => Style::default().fg(Color::Yellow),
            _ => Style::default(),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);

    match app.mode() {
        Mode::EnterCommand | Mode::CommandParameters => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.current_input().width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
        _ =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}
    }

    let mut messages: Vec<ListItem> = Vec::new();

    if let Some(error_message) = app.error_message() {
        let content = vec![Spans::from(Span::styled(
            format!("Error: {error_message}"),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))];
        messages.push(ListItem::new(content));
    }

    let include_number = !matches!(app.mode(), Mode::EnterCommand);
    if let Some(message_set) = app.current_display_set() {
        let content = vec![Spans::from(Span::styled(
            message_set.title.clone(),
            Style::default().add_modifier(Modifier::UNDERLINED),
        ))];
        messages.push(ListItem::new(content));

        for (i, m) in message_set.messages.iter().enumerate() {
            let message = if include_number {
                format!("{i}: {m}")
            } else {
                m.to_string()
            };
            let content = vec![Spans::from(Span::raw(message))];
            messages.push(ListItem::new(content));
        }
    }

    let current_page = app.current_page_display();
    let max_pages = app.num_pages();

    let body_title = match app.mode() {
        Mode::CommandParameters => "".to_string(),
        Mode::EnterCommand => "Enter Command".to_string(),
        Mode::Normal | Mode::Processing => {
            format!("Output (page {current_page} of {max_pages}) overflown text not shown, copy or export")
        }
    };

    let blocks =
        List::new(messages).block(Block::default().borders(Borders::ALL).title(body_title));
    f.render_widget(blocks, chunks[2]);
}
