mod app;
mod args;
mod dashboard;
mod ui;
mod window;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = args::Args::parse();

    if args.ghost_mode {
        if let Some(t) = args.time {
            app::run(&t)?;
        }
    } else {
        match args.time {
            Some(t) => {
                // We are the user's terminal -> Spawn the window
                let term_name = window::detect_terminal(args.terminal);
                window::spawn_ghost_window(&term_name, &t);
            }
            None => {
                //if no parameters found run the dashboard
                dashboard::run()?;
            }
        }
    }

    Ok(())
}
