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
    widgets::{BarChart, Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::{
    errors::InteractiveError, persistence::EventStore, projections::ListenTrackerRepository,
};

use unicode_width::UnicodeWidthStr;

use super::{App, AppMode, AppState};

pub async fn full_ui(
    store: Arc<dyn EventStore>,
    repository: Arc<Mutex<dyn ListenTrackerRepository>>,
) -> Result<(), InteractiveError> {
    let mut app = App::new(store, repository);
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

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        // Poll for an event, we do this so that we don't block on waiting for an event.
        // If we blocked, then we wouldn't tick the app to the next stage.
        if poll(Duration::from_millis(15))? {
            if let Event::Key(key) = event::read()? {
                match app.state.mode {
                    AppMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            app.start_command_input();
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Char('c') => {
                            app.copy_to_clipboard();
                        }
                        KeyCode::Char('w') => {
                            app.export_to_file().await;
                        }
                        _ => {}
                    },
                    AppMode::EnterCommand => match key.code {
                        KeyCode::Enter => {
                            app.command_name_entered();
                        }
                        KeyCode::Char(c) => {
                            app.state.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.state.input.pop();
                        }
                        KeyCode::Esc => {
                            app.cancel_command();
                        }
                        KeyCode::Tab => {
                            app.autocomplete_command_name();
                        }
                        _ => {}
                    },
                    AppMode::CommandParameters => match key.code {
                        KeyCode::Enter => {
                            app.advance_command_input();
                        }
                        KeyCode::Char(c) => {
                            app.state.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.state.input.pop();
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

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
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

    let (msg, style) = match app.state.mode {
        AppMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to enter command, "),
                Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to copy current output, "),
                Span::styled("w", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to write current output to files."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        AppMode::EnterCommand => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to execute the command"),
            ],
            Style::default(),
        ),
        AppMode::CommandParameters => {
            if let Some(spec) = app.state.command_parameter_inputs.get(0) {
                (vec![Span::raw(spec.description())], Style::default())
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

    let input = Paragraph::new(app.state.input.as_ref())
        .style(match app.state.mode {
            AppMode::EnterCommand | AppMode::CommandParameters => {
                Style::default().fg(Color::Yellow)
            }
            _ => Style::default(),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);

    match app.state.mode {
        AppMode::EnterCommand | AppMode::CommandParameters => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.state.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
        _ =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}
    }

    let mut messages: Vec<ListItem> = Vec::new();

    if let Some(error_message) = &app.state.error_message {
        let content = vec![Spans::from(Span::styled(
            format!("Error: {error_message}"),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))];
        messages.push(ListItem::new(content));
    }

    let include_number = !matches!(app.state.mode, AppMode::EnterCommand);

    for message_set in app.state.display_sets().iter() {
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

    let messages = List::new(messages).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Messages (overflown text not shown, copy or write to file to see)"),
    );

    let data = app.chart_data();

    let barchart = BarChart::default()
        .block(Block::default().title("Listens").borders(Borders::ALL))
        .data(&data)
        .bar_width(9)
        .bar_style(Style::default().fg(Color::Yellow))
        .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));
    f.render_widget(barchart, chunks[2]);
}
