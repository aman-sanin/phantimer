mod app;
mod args;
mod dashboard;
mod notification;
mod ui;
mod window;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = args::Args::parse();

    if args.ghost_mode {
        if args.pomodoro {
            app::run(None, true)?;
        } else if let Some(ref t) = args.time {
            app::run(Some(t), false)?;
        }
    } else {
        let term_name = window::detect_terminal(args.terminal);
        if let Some(ref t) = args.time {
            window::spawn_ghost_window(&term_name, Some(t), false);
        } else if args.pomodoro {
            window::spawn_ghost_window(&term_name, None, true);
        } else {
            dashboard::run()?;
        }
    }

    Ok(())
}
