use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Amount of time (e.g., 10s, 5m, 1h)
    pub time: Option<String>,

    /// Specify terminal emulator
    #[arg(short = 'T', long)]
    pub terminal: Option<String>,

    /// Activate Pomodoro session
    #[arg(short = 'p', long)]
    pub pomodoro: bool,

    /// Custom Pomodoro work session duration (e.g., 25m)
    #[arg(short = 'W', long)]
    pub work: Option<String>,

    /// Custom Pomodoro short break duration (e.g., 5m)
    #[arg(short = 'S', long)]
    pub short_break: Option<String>,

    /// Custom Pomodoro long break duration (e.g., 15m)
    #[arg(short = 'L', long)]
    pub long_break: Option<String>,

    /// Total number of rounds in a Pomodoro set
    #[arg(short = 'r', long)]
    pub rounds: Option<u32>,

    /// Number of rounds before a long break kicks in
    #[arg(short = 'i', long)]
    pub interval: Option<u32>,

    /// Internal flag: activates the TUI mode
    #[arg(long, hide = true)]
    pub ghost_mode: bool,
}
