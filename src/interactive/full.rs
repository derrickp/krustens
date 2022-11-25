use std::{io, str::FromStr, sync::Arc};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};

use crate::{
    errors::InteractiveError, persistence::EventStore, projections::statistics::EventProcessor,
};

use unicode_width::UnicodeWidthStr;

use super::{app::AppCommandName, AppMode, AppState};

pub async fn full_ui(store: Arc<impl EventStore>) -> Result<(), InteractiveError> {
    println!("Loading...");
    let mut processor = EventProcessor::default();
    let event_stream = store.get_events("listens".to_string()).await.unwrap();
    for event in event_stream.events.iter() {
        processor.process_event(event);
    }
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
    let app = AppState::default();
    let res = run_app(&mut terminal, app, &processor);

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
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: AppState,
    processor: &EventProcessor,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Event::Key(key) = event::read()? {
            match app.mode {
                AppMode::Normal => match key.code {
                    KeyCode::Char('c') => {
                        app.error_message = None;
                        app.command_name = None;
                        app.command_parameters = None;
                        app.mode = AppMode::EnterCommand;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }
                    _ => {}
                },
                AppMode::EnterCommand => match key.code {
                    KeyCode::Enter => {
                        app.error_message = None;
                        let text: String = app.input.drain(..).collect();
                        match AppCommandName::from_str(&text) {
                            Ok(it) => {
                                app.command_steps = it.parameters();
                                app.command_name = Some(it);
                                app.mode = AppMode::CommandParameters;
                            }
                            Err(_) => {
                                app.mode = AppMode::Normal;
                                app.error_message = Some("Unknown command name".to_string());
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.error_message = None;
                        app.mode = AppMode::Normal;
                    }
                    _ => {}
                },
                AppMode::CommandParameters => match key.code {
                    KeyCode::Enter => {
                        let text: String = app.input.drain(..).collect();
                        let spec = app.command_steps.remove(0);
                        match app.insert_command_parameter(&text, &spec) {
                            Ok(_) => {
                                if app.command_steps.is_empty() {
                                    app.run_command(processor);
                                    app.mode = AppMode::Normal;
                                }
                            }
                            Err(e) => {
                                app.error_message = Some(e.to_string());
                                app.mode = AppMode::Normal;
                                app.command_steps.clear();
                                app.command_name = None;
                                app.command_parameters = None;
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.error_message = None;
                        app.input.clear();
                        app.command_name = None;
                        app.command_steps.clear();
                        app.command_parameters = None;
                        app.mode = AppMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &AppState) {
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

    let (msg, style) = match app.mode {
        AppMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to enter command to run."),
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
            if let Some(spec) = app.command_steps.get(0) {
                (vec![Span::raw(spec.description())], Style::default())
            } else {
                (Vec::new(), Style::default())
            }
        }
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input = Paragraph::new(app.input.as_ref())
        .style(match app.mode {
            AppMode::Normal => Style::default(),
            AppMode::EnterCommand | AppMode::CommandParameters => {
                Style::default().fg(Color::Yellow)
            }
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);

    match app.mode {
        AppMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        AppMode::EnterCommand | AppMode::CommandParameters => {
            // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
            f.set_cursor(
                // Put cursor past the end of the input text
                chunks[1].x + app.input.width() as u16 + 1,
                // Move one line down, from the border to the input line
                chunks[1].y + 1,
            )
        }
    }

    let mut messages: Vec<ListItem> = Vec::new();

    if let Some(error_message) = &app.error_message {
        let content = vec![Spans::from(Span::styled(
            format!("Error: {}", error_message),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))];
        messages.push(ListItem::new(content));
    }

    for message_set in app.display_sets().iter() {
        let content = vec![Spans::from(Span::styled(
            message_set.title.clone(),
            Style::default().add_modifier(Modifier::UNDERLINED),
        ))];
        messages.push(ListItem::new(content));

        for (i, m) in message_set.messages.iter().enumerate() {
            let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
            messages.push(ListItem::new(content));
        }
    }

    let messages =
        List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    f.render_widget(messages, chunks[2]);
}
