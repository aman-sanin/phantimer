use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph},
};

pub fn render(f: &mut Frame, remaining_secs: u64, total_secs: u64) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(area);

    let time_str = format_time(remaining_secs);

    let text = Paragraph::new(time_str)
        .alignment(Alignment::Center) // Horizontal Center
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    //Center the Text Vertically (The Manual Way)
    let text_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Top Padding (Empty space)
            Constraint::Min(0),    // The actual text area
        ])
        .split(chunks[0])[1]; // We select the second chunk (the text area)

    // Render the Text
    f.render_widget(text, text_area);

    // Prepare and Render the Gauge
    let ratio = if total_secs > 0 {
        remaining_secs as f64 / total_secs as f64
    } else {
        0.0
    };

    // Dynamic Color Logic
    let gauge_color = if ratio < 0.2 {
        Color::Red
    } else {
        Color::Green
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::TOP)) // Only draw a line ABOVE the gauge
        .gauge_style(Style::default().fg(gauge_color))
        .ratio(ratio)
        .label(format!("{:.0}%", ratio * 100.0));

    f.render_widget(gauge, chunks[1]);
}

// Helper: Formats seconds into 00:00:00
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
