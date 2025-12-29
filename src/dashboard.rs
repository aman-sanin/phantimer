use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, widgets::*};
use std::{io::stdout, time::Duration};

use crate::window; // To launch the ghost window later

// 1. The State Machine
// We need to track what part of the UI is "Active"
#[derive(PartialEq)]
enum Focus {
    Input,
    Presets,
}

struct DashboardApp {
    input_text: String,
    // Presets data
    presets: Vec<(&'static str, &'static str)>, // (Name, Duration)
    selected_preset: usize,
    // Current focus state
    focus: Focus,
}

impl DashboardApp {
    fn new() -> Self {
        Self {
            input_text: String::new(),
            presets: vec![
                ("Pomodoro", "25m"),
                ("Short Break", "5m"),
                ("Long Break", "15m"),
                ("Meeting", "1h"),
                ("Standup", "15m"),
            ],
            selected_preset: 0,
            focus: Focus::Input, // Start with cursor in the input box
        }
    }

    // Navigation Logic
    fn next_preset(&mut self) {
        if self.selected_preset < self.presets.len() - 1 {
            self.selected_preset += 1;
        }
    }

    fn previous_preset(&mut self) {
        if self.selected_preset > 0 {
            self.selected_preset -= 1;
        }
    }
}

pub fn run() -> Result<()> {
    // Standard TUI Setup (Same as app.rs)
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = DashboardApp::new();

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match app.focus {
                    // --- MODE A: TYPING ---
                    Focus::Input => match key.code {
                        KeyCode::Enter => {
                            if !app.input_text.is_empty() {
                                // Close Dashboard -> Launch Timer
                                disable_raw_mode()?;
                                execute!(std::io::stdout(), LeaveAlternateScreen)?;
                                window::spawn_ghost_window("foot", &app.input_text);
                                return Ok(());
                            }
                        }
                        KeyCode::Char(c) => app.input_text.push(c),
                        KeyCode::Backspace => {
                            app.input_text.pop();
                        }
                        KeyCode::Esc => return Ok(()), // Quit
                        KeyCode::Tab | KeyCode::Down => app.focus = Focus::Presets, // Switch focus
                        _ => {}
                    },

                    // --- MODE B: SELECTING ---
                    Focus::Presets => match key.code {
                        KeyCode::Up | KeyCode::Char('k') => app.previous_preset(),
                        KeyCode::Down | KeyCode::Char('j') => app.next_preset(),
                        KeyCode::Enter => {
                            // Launch Selected Preset
                            let duration = app.presets[app.selected_preset].1;
                            disable_raw_mode()?;
                            execute!(std::io::stdout(), LeaveAlternateScreen)?;
                            window::spawn_ghost_window("foot", duration);
                            return Ok(());
                        }
                        KeyCode::Esc | KeyCode::Char('q') => return Ok(()),
                        KeyCode::Tab | KeyCode::BackTab => app.focus = Focus::Input, // Switch focus back
                        _ => {}
                    },
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut DashboardApp) {
    let area = f.area();

    // Layout:
    // 1. Header (Title)
    // 2. Input Box
    // 3. Presets List
    // 4. Footer (Help)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Input
            Constraint::Min(0),    // List (Takes rest)
            Constraint::Length(1), // Footer
        ])
        .split(area);

    // --- 1. HEADER ---
    let title = Paragraph::new("PHANTIMER")
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(title, chunks[0]);

    // --- 2. INPUT BOX ---
    // Dynamic Styling: If focused, turn Border Cyan. Else, Gray.
    let input_style = match app.focus {
        Focus::Input => Style::default().fg(Color::Cyan),
        Focus::Presets => Style::default().fg(Color::DarkGray),
    };

    let input = Paragraph::new(app.input_text.as_str())
        .style(match app.focus {
            Focus::Input => Style::default().fg(Color::White),
            _ => Style::default().fg(Color::DarkGray),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Custom Duration ")
                .border_style(input_style),
        );
    f.render_widget(input, chunks[1]);

    // --- 3. PRESETS LIST ---
    let items: Vec<ListItem> = app
        .presets
        .iter()
        .map(|(name, time)| {
            let content = format!("{:<15} ({})", name, time);
            ListItem::new(content)
        })
        .collect();

    // Highlighting Logic
    let list_style = match app.focus {
        Focus::Presets => Style::default().fg(Color::Cyan),
        Focus::Input => Style::default().fg(Color::DarkGray),
    };

    // The list itself
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Presets ")
                .border_style(list_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We need a 'ListState' to render the selection.
    // Since we are managing index manually in `app.selected_preset`, we create a temp state here.
    let mut state = ListState::default();
    state.select(Some(app.selected_preset));

    f.render_stateful_widget(list, chunks[2], &mut state);

    // --- 4. FOOTER ---
    let help_text = match app.focus {
        Focus::Input => "Type duration (e.g. 10m) • <Enter> Start • <Tab> Presets",
        Focus::Presets => "↑/↓ Navigate • <Enter> Select • <Tab> Custom Input",
    };
    let footer = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[3]);
}
