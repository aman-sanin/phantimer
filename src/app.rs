use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::prelude::*;
use std::{
    io::stdout,
    time::{Duration, Instant},
};

use crate::ui; // Import our UI module

pub fn run(time_str: &str) -> Result<()> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Parse Time
    let total_duration_secs = parse_duration(time_str);
    let start_time = Instant::now();
    let target_duration = Duration::from_secs(total_duration_secs);

    // 3. Main Loop
    loop {
        let elapsed = start_time.elapsed();
        let remaining = if elapsed < target_duration {
            target_duration - elapsed
        } else {
            Duration::from_secs(0)
        };
        let remaining_secs = remaining.as_secs();

        // Draw UI
        terminal.draw(|f| {
            ui::render(f, remaining_secs, total_duration_secs);
        })?;

        // Handle Input
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    break;
                }
            }
        }

        // Auto-close
        if remaining_secs == 0 {
            std::thread::sleep(Duration::from_secs(2));
            break;
        }
    }

    // 4. Cleanup
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn parse_duration(time_str: &str) -> u64 {
    let len = time_str.len();
    if len < 2 {
        return 0;
    }
    let (num_part, unit) = time_str.split_at(len - 1);
    let number: u64 = num_part.parse().unwrap_or(0);
    match unit {
        "s" => number,
        "m" => number * 60,
        "h" => number * 60 * 60,
        _ => number,
    }
}
