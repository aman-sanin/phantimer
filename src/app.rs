//src/app.rs
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

use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PomoStage {
    Work(u32),
    ShortBreak(u32),
    LongBreak,
}

impl PomoStage {
    pub fn name(&self) -> String {
        match self {
            PomoStage::Work(idx) => format!("🍅 Work [{}/4]", idx),
            PomoStage::ShortBreak(idx) => format!("☕ Break [{}/3]", idx),
            PomoStage::LongBreak => "🎉 Long Break".to_string(),
        }
    }

    pub fn duration_secs(&self) -> u64 {
        match self {
            PomoStage::Work(_) => 25 * 60,
            PomoStage::ShortBreak(_) => 5 * 60,
            PomoStage::LongBreak => 15 * 60,
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            PomoStage::Work(idx) => {
                if *idx < 4 {
                    Some(PomoStage::ShortBreak(*idx))
                } else {
                    Some(PomoStage::LongBreak)
                }
            }
            PomoStage::ShortBreak(idx) => Some(PomoStage::Work(*idx + 1)),
            PomoStage::LongBreak => None,
        }
    }
}

pub struct TimerState {
    pub title: String,
    pub remaining_secs: u64,
    pub total_secs: u64,
    pub is_paused: bool,
    pub is_pomodoro: bool,
    pub current_stage: Option<PomoStage>,
    pub message: Option<String>,
}

pub fn run(time_str: Option<&str>, is_pomodoro: bool) -> Result<()> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 2. Initialize State
    let mut state = if is_pomodoro {
        let stage = PomoStage::Work(1);
        TimerState {
            title: stage.name(),
            remaining_secs: stage.duration_secs(),
            total_secs: stage.duration_secs(),
            is_paused: false,
            is_pomodoro: true,
            current_stage: Some(stage),
            message: None,
        }
    } else {
        let total_secs = parse_duration(time_str.unwrap_or("0s"));
        TimerState {
            title: "Timer".to_string(),
            remaining_secs: total_secs,
            total_secs,
            is_paused: false,
            is_pomodoro: false,
            current_stage: None,
            message: None,
        }
    };

    // Send initial notification if we are starting Pomodoro
    if is_pomodoro {
        crate::notification::send("🍅 Focus Session 1 Started", "Time to get down to work! Focus for 25 minutes.");
    }

    let mut last_tick = Instant::now();
    let mut last_second_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    // 3. Main Loop
    loop {
        // Draw UI
        terminal.draw(|f| {
            ui::render(f, &state);
        })?;

        // Handle Input
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        break;
                    }
                    KeyCode::Char(' ') => {
                        // Toggle Pause
                        state.is_paused = !state.is_paused;
                        state.message = if state.is_paused {
                            Some("PAUSED • [Space] Resume".to_string())
                        } else {
                            None
                        };
                        last_second_tick = Instant::now(); // Reset second tick on toggle
                    }
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        // Precise 1-second ticks
        if last_second_tick.elapsed() >= Duration::from_secs(1) {
            if !state.is_paused && state.remaining_secs > 0 {
                state.remaining_secs = state.remaining_secs.saturating_sub(1);
            }
            last_second_tick = Instant::now();
        }

        // Handle Stage Completion / Timer Finish
        if state.remaining_secs == 0 {
            if state.is_pomodoro {
                if let Some(current) = state.current_stage {
                    if let Some(next_stage) = current.next() {
                        // Send notification
                        let (title, body) = match next_stage {
                            PomoStage::Work(idx) => (
                                format!("🍅 Work Session {}", idx),
                                "Time to focus! Back to work.".to_string(),
                            ),
                            PomoStage::ShortBreak(idx) => (
                                format!("☕ Break Session {}", idx),
                                "Great work! Enjoy a 5-minute break.".to_string(),
                            ),
                            PomoStage::LongBreak => (
                                "🎉 Long Break".to_string(),
                                "Fantastic job! Take a well-deserved 15-minute break.".to_string(),
                            ),
                        };
                        crate::notification::send(&title, &body);

                        // Update State for next stage
                        state.current_stage = Some(next_stage);
                        state.title = next_stage.name();
                        state.remaining_secs = next_stage.duration_secs();
                        state.total_secs = next_stage.duration_secs();
                        state.is_paused = true; // Start in paused state so user can prepare
                        state.message = Some("[Space] Start Next Session".to_string());
                        last_second_tick = Instant::now();
                    } else {
                        // Pomodoro set fully complete!
                        crate::notification::send("🎉 Pomodoro Complete!", "Excellent effort! You have completed all sessions.");
                        std::thread::sleep(Duration::from_secs(3));
                        break;
                    }
                }
            } else {
                // Normal timer completed
                crate::notification::send("🔔 Timer Finished", "Your countdown timer has completed.");
                std::thread::sleep(Duration::from_secs(2));
                break;
            }
        }
    }

    // 4. Cleanup
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn parse_duration(time_str: &str) -> u64 {
    if let Ok(d) = duration_str::parse(time_str) {
        d.as_secs()
    } else {
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
}
