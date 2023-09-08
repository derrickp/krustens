use std::{io, sync::Arc, time::Duration};

use crossterm::{
    event::{self, poll, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Backend, Constraint, CrosstermBackend, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{BarChart, Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use tokio::sync::Mutex;

use crate::{
    app::{Application, MessageSet, Mode, Output},
    errors::InteractiveError,
    persistence::{EventStore, StateStore},
    projections::ListenTrackerRepository,
};

use unicode_width::UnicodeWidthStr;

pub async fn full_ui(
    store: Arc<Mutex<dyn EventStore>>,
    state_store: Arc<Mutex<dyn StateStore>>,
    repository: Arc<Mutex<dyn ListenTrackerRepository>>,
) -> Result<(), InteractiveError> {
    let mut app = Application::new(store, repository, state_store);
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
                        KeyCode::Up => {
                            app.previous_command();
                        }
                        KeyCode::Down => {
                            app.next_command();
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
                            app.command_input_entered();
                        }
                        KeyCode::Tab => {
                            app.autocomplete_command_parameter();
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
    let mut text = Text::from(Line::from(msg));
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

    match app.current_output() {
        Some(Output::MessageSet(message_set)) => render_message_set(
            f,
            chunks[2],
            &message_set,
            app.error_message(),
            app.mode(),
            app.current_page_display(),
            app.num_pages(),
        ),
        Some(Output::BarChart(bar_chart)) => {
            render_chart(
                f,
                chunks[2],
                bar_chart,
                app.current_page_display(),
                app.num_pages(),
            );
        }
        None => render_empty(f, chunks[2], app.error_message(), app.mode()),
    }
}

fn render_message_set<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    message_set: &MessageSet,
    error_message: &Option<String>,
    mode: &Mode,
    current_page: usize,
    total_pages: usize,
) {
    let mut messages: Vec<ListItem> = Vec::new();
    if let Some(error_message) = error_message {
        let content = vec![Line::from(Span::styled(
            format!("Error: {error_message}"),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))];
        messages.push(ListItem::new(content));
    }

    let include_number = !matches!(mode, Mode::EnterCommand);

    for (i, m) in message_set.messages().iter().enumerate() {
        let message = if include_number {
            format!("{i}: {m}")
        } else {
            m.to_string()
        };
        let content = vec![Line::from(Span::raw(message))];
        messages.push(ListItem::new(content));
    }

    let body_title = match mode {
        Mode::CommandParameters => "".to_string(),
        Mode::EnterCommand => "Enter Command".to_string(),
        Mode::Normal | Mode::Processing => {
            format!(
                "{} page {current_page} of {total_pages} overflow not shown, copy or export to see",
                message_set.title()
            )
        }
    };

    let blocks =
        List::new(messages).block(Block::default().borders(Borders::ALL).title(body_title));
    f.render_widget(blocks, chunk);
}

fn render_empty<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    error_message: &Option<String>,
    mode: &Mode,
) {
    let mut messages: Vec<ListItem> = Vec::new();
    if let Some(error_message) = error_message {
        let content = vec![Line::from(Span::styled(
            format!("Error: {error_message}"),
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))];
        messages.push(ListItem::new(content));
    }

    let body_title = match mode {
        Mode::CommandParameters => "".to_string(),
        Mode::EnterCommand => "Enter Command".to_string(),
        Mode::Normal | Mode::Processing => "Output".to_string(),
    };

    let blocks =
        List::new(messages).block(Block::default().borders(Borders::ALL).title(body_title));
    f.render_widget(blocks, chunk);
}

fn render_chart<B: Backend>(
    f: &mut Frame<B>,
    chunk: Rect,
    bar_chart_data: crate::app::BarChart,
    current_page: usize,
    total_pages: usize,
) {
    let data: Vec<(&str, u64)> = bar_chart_data
        .data_points()
        .iter()
        .map(|data_point| (data_point.x(), data_point.y()))
        .collect();
    let title = format!(
        "{} page {current_page} of {total_pages}",
        bar_chart_data.title()
    );
    let bar_chart = BarChart::default()
        .block(Block::default().title(title).borders(Borders::ALL))
        .data(&data)
        .bar_width(9)
        .bar_style(Style::default().fg(Color::Yellow))
        .value_style(Style::default().fg(Color::Black).bg(Color::Yellow));
    f.render_widget(bar_chart, chunk);
}
