use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph},
};
use crate::app::{TimerState, PomoStage};

pub fn render(f: &mut Frame, state: &TimerState) {
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

    // --- 1. TITLE / HEADER ---
    let title_style = if state.is_paused {
        Style::default().fg(Color::DarkGray).add_modifier(Modifier::DIM)
    } else if state.is_pomodoro {
        match state.current_stage {
            Some(PomoStage::Work(_)) => Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD),
            _ => Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD),
        }
    } else {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    };

    let title_para = Paragraph::new(state.title.as_str())
        .alignment(Alignment::Center)
        .style(title_style);
    f.render_widget(title_para, chunks[0]);

    // --- 2. COUNTDOWN & STATUS MSG ---
    let time_str = format_time(state.remaining_secs);
    
    // Choose dynamic color schemes
    let time_color = if state.is_paused {
        Color::DarkGray
    } else if state.is_pomodoro {
        match state.current_stage {
            Some(PomoStage::Work(_)) => Color::Red,
            _ => Color::Green,
        }
    } else {
        Color::Cyan
    };

    // Sub-layout inside Middle chunk to center both clock and optional subtext vertically
    let middle_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top empty space padding
            Constraint::Length(1), // Large clock text
            Constraint::Length(1), // Optional blinking/small helper message
            Constraint::Min(0),    // Bottom padding
        ])
        .split(chunks[1]);

    // Render Large Clock centered
    let time_para = Paragraph::new(time_str)
        .alignment(Alignment::Center)
        .style(Style::default().fg(time_color).add_modifier(Modifier::BOLD));
    f.render_widget(time_para, middle_layout[1]);

    // Render helper status message if any
    if let Some(ref msg) = state.message {
        let msg_style = if state.is_paused && msg.contains("PAUSED") {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)
        };
        let msg_para = Paragraph::new(msg.as_str())
            .alignment(Alignment::Center)
            .style(msg_style);
        f.render_widget(msg_para, middle_layout[2]);
    }

    // --- 3. BOTTOM PROGRESS BAR ---
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
            Some(PomoStage::Work(_)) => Color::LightRed,
            _ => Color::LightGreen,
        }
    } else {
        Color::Green
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::TOP))
        .gauge_style(Style::default().fg(gauge_color))
        .ratio(ratio)
        .label(format!("{:.0}%", ratio * 100.0));

    f.render_widget(gauge, chunks[2]);
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
