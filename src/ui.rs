use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph},
};
use crate::app::{TimerState, PomoStage};

pub fn render(f: &mut Frame, state: &TimerState, config: &crate::config::Config) {
    let area = f.area();

    // Split layout vertically:
    // 1. Header (Height 1) - showing the session name or "Timer"
    // 2. Middle (Height Min 0) - contains centering layout for remaining time and status messages
    // 3. Footer (Height 2) - bottom gauge bar (including the top divider)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title line
            Constraint::Min(0),    // Main timer section
            Constraint::Length(2), // Bottom gauge line + progress bar
        ])
        .split(area);

    // Resolve color configuration values
    let (color_timer, color_work, color_break, color_paused) = if let Some(ref colors) = config.colors {
        (
            colors.timer_text.as_ref().map(|c| crate::config::parse_color(c)).unwrap_or(Color::Cyan),
            colors.work_session.as_ref().map(|c| crate::config::parse_color(c)).unwrap_or(Color::Red),
            colors.break_session.as_ref().map(|c| crate::config::parse_color(c)).unwrap_or(Color::Green),
            colors.paused_text.as_ref().map(|c| crate::config::parse_color(c)).unwrap_or(Color::Yellow),
        )
    } else {
        (Color::Cyan, Color::Red, Color::Green, Color::Yellow)
    };

    // --- 1. TITLE / HEADER ---
    let title_style = if state.is_done {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    } else if state.is_paused {
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)
    } else if state.is_pomodoro {
        match state.current_stage {
            Some(PomoStage::Work) => Style::default().fg(color_work).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(color_break).add_modifier(Modifier::BOLD),
        }
    } else {
        Style::default().fg(color_timer).add_modifier(Modifier::BOLD)
    };

    let title_para = Paragraph::new(state.title.as_str())
        .alignment(Alignment::Center)
        .style(title_style);
    f.render_widget(title_para, chunks[0]);

    // --- 2. COUNTDOWN & STATUS MSG ---
    let display_secs = if state.is_stopwatch {
        state.elapsed_secs
    } else {
        state.remaining_secs
    };
    let time_str = format_time(display_secs);
    
    // Choose dynamic color schemes
    let time_color = if state.is_done {
        Color::Green
    } else if state.is_paused {
        Color::DarkGray
    } else if state.is_pomodoro {
        match state.current_stage {
            Some(PomoStage::Work) => color_work,
            _ => color_break,
        }
    } else {
        color_timer
    };

    let big_lines = if state.is_done {
        Some(format_big_done())
    } else {
        Some(format_big_time(&time_str))
    };
    let big_width = big_lines.as_ref().map(|l| l[0].chars().count() as u16).unwrap_or(0);
    let use_big = chunks[1].height >= 4 && chunks[1].width >= big_width + 2;

    if use_big {
        let lines = big_lines.unwrap();
        let big_text = lines.join("\n");
        let available_h = chunks[1].height;
        let needed_h = if state.message.is_some() { 4 } else { 3 };
        let top_pad = available_h.saturating_sub(needed_h) / 2;

        let mut constraints = vec![
            Constraint::Length(top_pad),
            Constraint::Length(3),
        ];
        if state.message.is_some() {
            constraints.push(Constraint::Length(1));
        }
        constraints.push(Constraint::Min(0));

        let middle_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(chunks[1]);

        let time_para = Paragraph::new(big_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(time_color).add_modifier(Modifier::BOLD));
        f.render_widget(time_para, middle_layout[1]);

        if let Some(ref msg) = state.message {
            let msg_style = if state.is_done {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if state.is_paused && (msg.contains("PAUSED") || msg.contains("RESET")) {
                Style::default().fg(color_paused).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
            };
            let msg_para = Paragraph::new(msg.as_str())
                .alignment(Alignment::Center)
                .style(msg_style);
            f.render_widget(msg_para, middle_layout[2]);
        }
    } else {
        let middle_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(chunks[1].height.saturating_sub(2) / 2),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(chunks[1]);

        let display_text = if state.is_done {
            "✔ DONE".to_string()
        } else {
            time_str
        };
        let time_para = Paragraph::new(display_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(time_color).add_modifier(Modifier::BOLD));
        f.render_widget(time_para, middle_layout[1]);

        if let Some(ref msg) = state.message {
            let msg_style = if state.is_done {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else if state.is_paused && (msg.contains("PAUSED") || msg.contains("RESET")) {
                Style::default().fg(color_paused).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
            };
            let msg_para = Paragraph::new(msg.as_str())
                .alignment(Alignment::Center)
                .style(msg_style);
            f.render_widget(msg_para, middle_layout[2]);
        }
    }

    // --- 3. BOTTOM PROGRESS BAR ---
    let gauge = if state.is_done {
        Gauge::default()
            .block(Block::default().borders(Borders::TOP))
            .gauge_style(Style::default().fg(Color::Green))
            .ratio(1.0)
            .label("DONE")
    } else if state.is_stopwatch {
        let gauge_color = if state.is_paused {
            Color::DarkGray
        } else {
            color_timer
        };
        let gauge_label = if state.is_paused { "PAUSED" } else { "" };
        Gauge::default()
            .block(Block::default().borders(Borders::TOP))
            .gauge_style(Style::default().fg(gauge_color))
            .ratio(1.0)
            .label(gauge_label)
    } else {
        let ratio = if state.total_secs > 0 {
            state.remaining_secs as f64 / state.total_secs as f64
        } else {
            0.0
        };

        let gauge_color = if state.is_paused {
            Color::DarkGray
        } else if ratio < 0.2 {
            Color::Red // critical alert state
        } else if state.is_pomodoro {
            match state.current_stage {
                Some(PomoStage::Work) => color_work,
                _ => color_break,
            }
        } else {
            color_break
        };

        Gauge::default()
            .block(Block::default().borders(Borders::TOP))
            .gauge_style(Style::default().fg(gauge_color))
            .ratio(ratio)
            .label(format!("{:.0}%", ratio * 100.0))
    };

    f.render_widget(gauge, chunks[2]);
}

fn get_big_digit_row(c: char, row: usize) -> &'static str {
    match (c, row) {
        ('0', 0) => "█▀█",
        ('0', 1) => "█ █",
        ('0', 2) => "█▄█",

        ('1', 0) => " █ ",
        ('1', 1) => " █ ",
        ('1', 2) => " █ ",

        ('2', 0) => "█▀█",
        ('2', 1) => " ▄▀",
        ('2', 2) => "█▄▄",

        ('3', 0) => "█▀█",
        ('3', 1) => " ▀█",
        ('3', 2) => "█▄█",

        ('4', 0) => "█ █",
        ('4', 1) => "█▄█",
        ('4', 2) => "  █",

        ('5', 0) => "█▀▀",
        ('5', 1) => "▀▀█",
        ('5', 2) => "▄▄█",

        ('6', 0) => "█▀▀",
        ('6', 1) => "█▀█",
        ('6', 2) => "█▄█",

        ('7', 0) => "▀▀█",
        ('7', 1) => "  █",
        ('7', 2) => "  █",

        ('8', 0) => "█▀█",
        ('8', 1) => "█▀█",
        ('8', 2) => "█▄█",

        ('9', 0) => "█▀█",
        ('9', 1) => "▀▀█",
        ('9', 2) => " ▄█",

        (':', 0) => " ",
        (':', 1) => "•",
        (':', 2) => "•",

        _ => "   ",
    }
}

fn format_big_time(time_str: &str) -> Vec<String> {
    let mut rows = vec![String::new(), String::new(), String::new()];
    for (i, c) in time_str.chars().enumerate() {
        if i > 0 {
            rows[0].push(' ');
            rows[1].push(' ');
            rows[2].push(' ');
        }
        rows[0].push_str(get_big_digit_row(c, 0));
        rows[1].push_str(get_big_digit_row(c, 1));
        rows[2].push_str(get_big_digit_row(c, 2));
    }
    rows
}

fn format_big_done() -> Vec<String> {
    vec![
        "█▀▄  █▀█  █▄ █  █▀▀".to_string(),
        "█ █  █ █  █ ▀█  █▀▀".to_string(),
        "▀▀   ▀▀▀  ▀  ▀  ▀▀▀".to_string(),
    ]
}

// Helper: Formats seconds into 00:00:00 or 00:00
fn format_time(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}
