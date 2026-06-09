use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{prelude::*, widgets::*};
use std::{io::stdout, time::Duration};

use crate::window;

#[derive(PartialEq)]
enum Focus {
    Input,
    Presets,
    Pomodoro,
}

#[derive(PartialEq)]
enum PomoField {
    Work,
    ShortBreak,
    LongBreak,
    Rounds,
    Interval,
    StartButton,
}

impl PomoField {
    fn next(&self) -> Self {
        match self {
            PomoField::Work => PomoField::ShortBreak,
            PomoField::ShortBreak => PomoField::LongBreak,
            PomoField::LongBreak => PomoField::Rounds,
            PomoField::Rounds => PomoField::Interval,
            PomoField::Interval => PomoField::StartButton,
            PomoField::StartButton => PomoField::Work,
        }
    }

    fn prev(&self) -> Self {
        match self {
            PomoField::Work => PomoField::StartButton,
            PomoField::ShortBreak => PomoField::Work,
            PomoField::LongBreak => PomoField::ShortBreak,
            PomoField::Rounds => PomoField::LongBreak,
            PomoField::Interval => PomoField::Rounds,
            PomoField::StartButton => PomoField::Interval,
        }
    }
}

struct DashboardApp {
    input_text: String,
    presets: Vec<(String, String)>,
    selected_preset: usize,
    focus: Focus,
    pomo_field: PomoField,
    pomo_work: String,
    pomo_short: String,
    pomo_long: String,
    pomo_rounds: String,
    pomo_interval: String,
}

impl DashboardApp {
    fn new(config: &crate::config::Config) -> Self {
        let mut presets = Vec::new();
        if let Some(ref config_presets) = config.presets {
            for (k, v) in config_presets {
                presets.push((k.clone(), v.clone()));
            }
        }
        if presets.is_empty() {
            presets = vec![
                ("Pomodoro".to_string(), "25m".to_string()),
                ("Short Break".to_string(), "5m".to_string()),
                ("Long Break".to_string(), "15m".to_string()),
                ("Meeting".to_string(), "1h".to_string()),
                ("Standup".to_string(), "15m".to_string()),
            ];
        }

        let (w, s, l, r, i) = if let Some(ref pomo) = config.pomodoro {
            (
                pomo.work.clone().unwrap_or_else(|| "25m".to_string()),
                pomo.short_break.clone().unwrap_or_else(|| "5m".to_string()),
                pomo.long_break.clone().unwrap_or_else(|| "15m".to_string()),
                pomo.rounds.unwrap_or(4).to_string(),
                pomo.long_break_interval.unwrap_or(4).to_string(),
            )
        } else {
            ("25m".to_string(), "5m".to_string(), "15m".to_string(), "4".to_string(), "4".to_string())
        };

        Self {
            input_text: String::new(),
            presets,
            selected_preset: 0,
            focus: Focus::Input,
            pomo_field: PomoField::Work,
            pomo_work: w,
            pomo_short: s,
            pomo_long: l,
            pomo_rounds: r,
            pomo_interval: i,
        }
    }

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

pub fn run(config: &crate::config::Config) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = DashboardApp::new(config);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                match app.focus {
                    // --- MODE A: INPUT BOX ---
                    Focus::Input => match key.code {
                        KeyCode::Enter => {
                            if !app.input_text.is_empty() {
                                disable_raw_mode()?;
                                execute!(std::io::stdout(), LeaveAlternateScreen)?;
                                let term = window::detect_terminal(None, config);
                                window::spawn_ghost_window(
                                    &term,
                                    Some(&app.input_text),
                                    false,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    config,
                                );
                                return Ok(());
                            }
                        }
                        KeyCode::Char(c) => app.input_text.push(c),
                        KeyCode::Backspace => {
                            app.input_text.pop();
                        }
                        KeyCode::Esc => break,
                        KeyCode::Tab | KeyCode::Down => app.focus = Focus::Presets,
                        _ => {}
                    },

                    // --- MODE B: PRESETS COLUMN ---
                    Focus::Presets => match key.code {
                        KeyCode::Up | KeyCode::Char('k') => app.previous_preset(),
                        KeyCode::Down | KeyCode::Char('j') => app.next_preset(),
                        KeyCode::Enter => {
                            let name = &app.presets[app.selected_preset].0;
                            let duration = &app.presets[app.selected_preset].1;
                            let is_pomo = name == "Pomodoro";
                            disable_raw_mode()?;
                            execute!(std::io::stdout(), LeaveAlternateScreen)?;
                            let term = window::detect_terminal(None, config);
                            let r = config.pomodoro.as_ref().and_then(|p| p.rounds);
                            let i = config.pomodoro.as_ref().and_then(|p| p.long_break_interval);
                            window::spawn_ghost_window(
                                &term,
                                if is_pomo { None } else { Some(duration) },
                                is_pomo,
                                None,
                                None,
                                None,
                                r,
                                i,
                                config,
                            );
                            return Ok(());
                        }
                        KeyCode::Esc | KeyCode::Char('q') => break,
                        KeyCode::Tab => app.focus = Focus::Pomodoro,
                        KeyCode::BackTab => app.focus = Focus::Input,
                        _ => {}
                    },

                    // --- MODE C: POMODORO COLUMN ---
                    Focus::Pomodoro => match key.code {
                        KeyCode::Up | KeyCode::Char('k') => app.pomo_field = app.pomo_field.prev(),
                        KeyCode::Down | KeyCode::Char('j') => app.pomo_field = app.pomo_field.next(),
                        KeyCode::Enter => {
                            disable_raw_mode()?;
                            execute!(std::io::stdout(), LeaveAlternateScreen)?;
                            let term = window::detect_terminal(None, config);
                            let rounds: Option<u32> = app.pomo_rounds.parse().ok();
                            let interval: Option<u32> = app.pomo_interval.parse().ok();
                            window::spawn_ghost_window(
                                &term,
                                None,
                                true,
                                Some(&app.pomo_work),
                                Some(&app.pomo_short),
                                Some(&app.pomo_long),
                                rounds,
                                interval,
                                config,
                            );
                            return Ok(());
                        }
                        KeyCode::Esc | KeyCode::Char('q') => break,
                        KeyCode::Tab => app.focus = Focus::Input,
                        KeyCode::BackTab => app.focus = Focus::Presets,
                        KeyCode::Char(c) => {
                            match app.pomo_field {
                                PomoField::Work => app.pomo_work.push(c),
                                PomoField::ShortBreak => app.pomo_short.push(c),
                                PomoField::LongBreak => app.pomo_long.push(c),
                                PomoField::Rounds => {
                                    if c.is_ascii_digit() {
                                        app.pomo_rounds.push(c);
                                    }
                                }
                                PomoField::Interval => {
                                    if c.is_ascii_digit() {
                                        app.pomo_interval.push(c);
                                    }
                                }
                                PomoField::StartButton => {}
                            }
                        }
                        KeyCode::Backspace => {
                            match app.pomo_field {
                                PomoField::Work => { app.pomo_work.pop(); }
                                PomoField::ShortBreak => { app.pomo_short.pop(); }
                                PomoField::LongBreak => { app.pomo_long.pop(); }
                                PomoField::Rounds => { app.pomo_rounds.pop(); }
                                PomoField::Interval => { app.pomo_interval.pop(); }
                                PomoField::StartButton => {}
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn ui(f: &mut Frame, app: &mut DashboardApp) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Input
            Constraint::Min(0),    // Columns
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
    let input_style = match app.focus {
        Focus::Input => Style::default().fg(Color::Cyan),
        _ => Style::default().fg(Color::DarkGray),
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

    // --- 3. MIDDLE AREA (Presets & Pomodoro side-by-side) ---
    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[2]);

    // --- PRESETS LIST ---
    let items: Vec<ListItem> = app
        .presets
        .iter()
        .map(|(name, time)| {
            let content = format!("{:<15} ({})", name, time);
            ListItem::new(content)
        })
        .collect();

    let presets_style = match app.focus {
        Focus::Presets => Style::default().fg(Color::Cyan),
        _ => Style::default().fg(Color::DarkGray),
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Presets ")
                .border_style(presets_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(Some(app.selected_preset));
    f.render_stateful_widget(list, middle_chunks[0], &mut state);

    // --- CUSTOM POMODORO PANEL ---
    let pomo_style = match app.focus {
        Focus::Pomodoro => Style::default().fg(Color::Cyan),
        _ => Style::default().fg(Color::DarkGray),
    };

    let pomo_block = Block::default()
        .borders(Borders::ALL)
        .title(" Custom Pomodoro ")
        .border_style(pomo_style);
    
    let pomo_inner_area = pomo_block.inner(middle_chunks[1]);
    f.render_widget(pomo_block, middle_chunks[1]);

    let pomo_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Work
            Constraint::Length(1), // Short Break
            Constraint::Length(1), // Long Break
            Constraint::Length(1), // Total Rounds
            Constraint::Length(1), // Long Break Int
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Start Button
        ])
        .split(pomo_inner_area);

    let draw_line = |f: &mut Frame, area: Rect, label: &str, val: &str, field: PomoField, current_field: &PomoField, focus: &Focus| {
        let is_selected = *focus == Focus::Pomodoro && field == *current_field;
        let prefix = if is_selected { ">> " } else { "   " };
        let style = if is_selected {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let text = format!("{}{:<18} {}", prefix, label, val);
        f.render_widget(Paragraph::new(text).style(style), area);
    };

    draw_line(f, pomo_layout[0], "Work Duration:", &app.pomo_work, PomoField::Work, &app.pomo_field, &app.focus);
    draw_line(f, pomo_layout[1], "Short Break:", &app.pomo_short, PomoField::ShortBreak, &app.pomo_field, &app.focus);
    draw_line(f, pomo_layout[2], "Long Break:", &app.pomo_long, PomoField::LongBreak, &app.pomo_field, &app.focus);
    draw_line(f, pomo_layout[3], "Total Rounds:", &app.pomo_rounds, PomoField::Rounds, &app.pomo_field, &app.focus);
    draw_line(f, pomo_layout[4], "Long Break Int:", &app.pomo_interval, PomoField::Interval, &app.pomo_field, &app.focus);

    // Start Button
    let button_field_style = if app.focus == Focus::Pomodoro && app.pomo_field == PomoField::StartButton {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let button_style = if app.focus == Focus::Pomodoro && app.pomo_field == PomoField::StartButton {
        Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    
    let start_button = Paragraph::new(" [ Start Pomodoro ] ")
        .alignment(Alignment::Center)
        .style(button_style)
        .block(Block::default().borders(Borders::ALL).border_style(button_field_style));
    f.render_widget(start_button, pomo_layout[6]);

    // --- 4. FOOTER ---
    let help_text = match app.focus {
        Focus::Input => "Type duration (e.g. 10m) • <Enter> Start • <Tab> Presets",
        Focus::Presets => "↑/↓ Navigate • <Enter> Select • <Tab> Pomodoro Settings • <Shift+Tab> Input",
        Focus::Pomodoro => match app.pomo_field {
            PomoField::StartButton => "↑/↓ Navigate • <Enter> Start Pomodoro • <Tab> Input • <Shift+Tab> Presets",
            _ => "Type value (e.g. 25m / 4) • <Enter> Start Pomodoro • ↑/↓ Navigate • <Tab> Input",
        }
    };
    let footer = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(footer, chunks[3]);
}
