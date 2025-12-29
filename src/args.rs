use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Amount of time (e.g., 10s, 5m, 1h)
    pub time: Option<String>,

    /// Specify terminal emulator
    #[arg(short = 'T', long)]
    pub terminal: Option<String>,

    /// Internal flag: activates the TUI mode
    #[arg(long, hide = true)]
    pub ghost_mode: bool,
}
