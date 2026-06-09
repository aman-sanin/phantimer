mod app;
mod args;
mod config;
mod dashboard;
mod notification;
mod ui;
mod window;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = args::Args::parse();
    let config = config::Config::load();

    if args.ghost_mode {
        app::run(
            args.time.as_deref(),
            args.pomodoro,
            args.work,
            args.short_break,
            args.long_break,
            args.rounds,
            args.interval,
            &config,
        )?;
    } else {
        let term_name = window::detect_terminal(args.terminal, &config);
        if let Some(ref t) = args.time {
            window::spawn_ghost_window(&term_name, Some(t), false, None, None, None, None, None, &config);
        } else if args.pomodoro {
            window::spawn_ghost_window(
                &term_name,
                None,
                true,
                args.work.as_deref(),
                args.short_break.as_deref(),
                args.long_break.as_deref(),
                args.rounds,
                args.interval,
                &config,
            );
        } else {
            dashboard::run(&config)?;
        }
    }

    Ok(())
}
