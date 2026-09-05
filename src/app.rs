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
    Work,
    ShortBreak,
    LongBreak,
}

impl PomoStage {
    pub fn duration_secs(&self, durs: PomoDurations) -> u64 {
        match self {
            PomoStage::Work => durs.work,
            PomoStage::ShortBreak => durs.short_break,
            PomoStage::LongBreak => durs.long_break,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PomoDurations {
    pub work: u64,
    pub short_break: u64,
    pub long_break: u64,
}

pub struct TimerState {
    pub title: String,
    pub remaining_secs: u64,
    pub total_secs: u64,
    pub elapsed_secs: u64,
    pub is_paused: bool,
    pub is_pomodoro: bool,
    pub is_stopwatch: bool,
    pub current_stage: Option<PomoStage>,
    pub current_round: u32,
    pub total_rounds: u32,
    pub long_break_interval: u32,
    pub message: Option<String>,
}

pub fn run(
    time_str: Option<&str>,
    is_pomodoro: bool,
    is_stopwatch: bool,
    work_dur: Option<String>,
    short_dur: Option<String>,
    long_dur: Option<String>,
    rounds: Option<u32>,
    interval: Option<u32>,
    config: &crate::config::Config,
) -> Result<()> {
    // 1. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Resolve custom Pomodoro durations & parameters if active
    let pomo_durs = if is_pomodoro {
        let work_secs = resolve_duration(
            work_dur.as_deref(),
            config.pomodoro.as_ref().and_then(|p| p.work.as_deref()),
            "25m",
        );
        let short_secs = resolve_duration(
            short_dur.as_deref(),
            config.pomodoro.as_ref().and_then(|p| p.short_break.as_deref()),
            "5m",
        );
        let long_secs = resolve_duration(
            long_dur.as_deref(),
            config.pomodoro.as_ref().and_then(|p| p.long_break.as_deref()),
            "15m",
        );
        let r = rounds.unwrap_or_else(|| config.pomodoro.as_ref().and_then(|p| p.rounds).unwrap_or(4));
        let i = interval.unwrap_or_else(|| config.pomodoro.as_ref().and_then(|p| p.long_break_interval).unwrap_or(4));
        Some((PomoDurations {
            work: work_secs,
            short_break: short_secs,
            long_break: long_secs,
        }, r, i))
    } else {
        None
    };

    // 2. Initialize State
    let mut state = if is_pomodoro {
        let (durs, r, i) = pomo_durs.unwrap();
        let stage = PomoStage::Work;
        let duration = stage.duration_secs(durs);
        TimerState {
            title: format!("🍅 Work [1/{}]", r),
            remaining_secs: duration,
            total_secs: duration,
            elapsed_secs: 0,
            is_paused: false,
            is_pomodoro: true,
            is_stopwatch: false,
            current_stage: Some(stage),
            current_round: 1,
            total_rounds: r,
            long_break_interval: i,
            message: None,
        }
    } else if is_stopwatch {
        TimerState {
            title: "⏱ Stopwatch".to_string(),
            remaining_secs: 0,
            total_secs: 0,
            elapsed_secs: 0,
            is_paused: false,
            is_pomodoro: false,
            is_stopwatch: true,
            current_stage: None,
            current_round: 0,
            total_rounds: 0,
            long_break_interval: 0,
            message: None,
        }
    } else {
        let total_secs = parse_duration(time_str.unwrap_or("0s"));
        TimerState {
            title: "Timer".to_string(),
            remaining_secs: total_secs,
            total_secs,
            elapsed_secs: 0,
            is_paused: false,
            is_pomodoro: false,
            is_stopwatch: false,
            current_stage: None,
            current_round: 0,
            total_rounds: 0,
            long_break_interval: 0,
            message: None,
        }
    };

    // Send initial notification if starting Pomodoro
    if is_pomodoro {
        crate::notification::send("🍅 Focus Session 1 Started", "Time to get down to work!");
    }

    let mut last_tick = Instant::now();
    let mut last_second_tick = Instant::now();
    let tick_rate = Duration::from_millis(250);

    // 3. Main Loop
    loop {
        // Draw UI
        terminal.draw(|f| {
            ui::render(f, &state, config);
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
                        last_second_tick = Instant::now();
                    }
                    KeyCode::Char('R') => {
                        // Reset current timer/stage/stopwatch
                        if state.is_stopwatch {
                            state.elapsed_secs = 0;
                            state.is_paused = true;
                            state.message = Some("RESET • [Space] Start".to_string());
                        } else {
                            state.remaining_secs = state.total_secs;
                            state.is_paused = true;
                            state.message = Some("RESET • [Space] Start".to_string());
                        }
                        last_second_tick = Instant::now();
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
            if !state.is_paused {
                if state.is_stopwatch {
                    state.elapsed_secs = state.elapsed_secs.saturating_add(1);
                } else if state.remaining_secs > 0 {
                    state.remaining_secs = state.remaining_secs.saturating_sub(1);
                }
            }
            last_second_tick = Instant::now();
        }

        // Handle Stage Completion / Timer Finish
        if !state.is_stopwatch && state.remaining_secs == 0 {
            if state.is_pomodoro {
                if let Some(current) = state.current_stage {
                    let durs = pomo_durs.unwrap().0;
                    match current {
                        PomoStage::Work => {
                            // Determine if we do a long break or a short break
                            let is_long_break = state.current_round % state.long_break_interval == 0;
                            let next_stage = if is_long_break { PomoStage::LongBreak } else { PomoStage::ShortBreak };
                            
                            let (title, body) = if is_long_break {
                                ("🎉 Long Break".to_string(), format!("Completed round {}! Take a long break.", state.current_round))
                            } else {
                                (format!("☕ Short Break"), format!("Completed round {}! Take a short break.", state.current_round))
                            };
                            crate::notification::send(&title, &body);

                            let next_secs = next_stage.duration_secs(durs);
                            state.current_stage = Some(next_stage);
                            state.title = match next_stage {
                                PomoStage::LongBreak => format!("🎉 Long Break [{}/{}]", state.current_round, state.total_rounds),
                                _ => format!("☕ Break [{}/{}]", state.current_round, state.total_rounds),
                            };
                            state.remaining_secs = next_secs;
                            state.total_secs = next_secs;
                            state.is_paused = true;
                            state.message = Some("[Space] Start Next Session".to_string());
                            last_second_tick = Instant::now();
                        }
                        PomoStage::ShortBreak | PomoStage::LongBreak => {
                            // Transition to next round Work
                            state.current_round += 1;
                            if state.current_round > state.total_rounds {
                                crate::notification::send("🎉 Pomodoro Complete!", "Excellent effort! You have completed all sessions.");
                                std::thread::sleep(Duration::from_secs(3));
                                break;
                            } else {
                                let next_stage = PomoStage::Work;
                                let next_secs = next_stage.duration_secs(durs);
                                crate::notification::send(
                                    &format!("🍅 Work Session {}", state.current_round),
                                    "Time to focus! Back to work.",
                                );

                                state.current_stage = Some(next_stage);
                                state.title = format!("🍅 Work [{}/{}]", state.current_round, state.total_rounds);
                                state.remaining_secs = next_secs;
                                state.total_secs = next_secs;
                                state.is_paused = true;
                                state.message = Some("[Space] Start Next Session".to_string());
                                last_second_tick = Instant::now();
                            }
                        }
                    }
                }
            } else {
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

fn resolve_duration(cli_val: Option<&str>, config_val: Option<&str>, default_str: &str) -> u64 {
    if let Some(c) = cli_val {
        parse_duration(c)
    } else if let Some(cfg) = config_val {
        parse_duration(cfg)
    } else {
        parse_duration(default_str)
    }
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
