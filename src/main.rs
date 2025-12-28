use clap::Parser;
use crossterm::cursor::MoveTo;
use crossterm::terminal::Clear;
use crossterm::{ExecutableCommand, cursor, terminal};
use std::env;
use std::io::{Write, stdout};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///Amount of time
    time: String,

    ///specify terminal emulator
    #[arg(short = 'T', long)]
    terminal: Option<String>,

    #[arg(long, hide = true)]
    ghost_mode: bool,
}

fn main() {
    let args = Args::parse();

    if args.ghost_mode {
        let mut total_seconds = parse_duration(&args.time);

        let mut stdout = stdout();
        stdout.execute(cursor::Hide).unwrap();

        //Countdown-loop
        while total_seconds > 0 {
            stdout.execute(MoveTo(0, 0)).unwrap();
            stdout.execute(Clear(terminal::ClearType::All)).unwrap();

            print!("Timer: {}s", total_seconds);
            stdout.flush().unwrap();

            std::thread::sleep(std::time::Duration::from_secs(1));
            total_seconds -= 1;
        }

        stdout.execute(MoveTo(0, 0)).unwrap();
        stdout.execute(Clear(terminal::ClearType::All)).unwrap();
        println!("Timer: DONE!");

        //keep window open for a moment to see Done!
        std::thread::sleep(std::time::Duration::from_secs(2));

        stdout.execute(cursor::Show).unwrap();
    } else {
        //---Launcher Logic---

        let term_name = detect_terminal(args.terminal);
        spawn_ghost_window(&term_name, &args.time);
    }
}

fn detect_terminal(user_arg: Option<String>) -> String {
    //priority1:user-given-terminal-flag
    if let Some(t) = user_arg {
        return t;
    }

    //priority2: terminal-env-var-set
    if let Some(env_term) = env::var("TERMINAL").ok() {
        return env_term;
    }

    //priority3:default-fallback-terminal
    "foot".to_string()
}

fn spawn_ghost_window(terminal: &str, time: &str) {
    let current_exe = std::env::current_exe().expect("Failed to get current executable path");
    let exe_path = current_exe.to_str().expect("Path contains invalid unicode");

    let (class_flag, class_name) = match terminal {
        "foot" => ("--app-id", "floating-timer"),
        "kitty" => ("--class", "floating-timer"),
        "alacritty" => ("--class", "floating-timer"),
        _ => ("--class", "floating-timer"),
    };

    //Dynamic-Hyprland-Rule
    //to set floating requirement
    if cfg!(target_os = "linux") {
        // Rule: Float
        let _ = std::process::Command::new("hyprctl")
            .args([
                "keyword",
                "windowrulev2",
                &format!("float, class:^({})$", class_name),
            ])
            .output();

        //Rule: Pinning
        let _ = std::process::Command::new("hyprctl")
            .args([
                "keyword",
                "windowrulev2",
                &format!("pin, class:^({})$", class_name),
            ])
            .output();

        // Rule: Size (Width x Height)
        let _ = std::process::Command::new("hyprctl")
            .args([
                "keyword",
                "windowrulev2",
                &format!("size 300 150, class:^({})$", class_name),
            ])
            .output();

        // Rule: Position (Top Right - optional but recommended)
        let _ = std::process::Command::new("hyprctl")
            .args([
                "keyword",
                "windowrulev2",
                &format!("move 100%-310 50, class:^({})$", class_name),
            ])
            .output();

        // Rule: No Border (Make it look like a pure widget)
        let _ = std::process::Command::new("hyprctl")
            .args([
                "keyword",
                "windowrulev2",
                &format!("noborder, class:^({})$", class_name),
            ])
            .output();
    }

    let mut cmd = std::process::Command::new(terminal);

    //terminal-specific-sizing
    if terminal == "foot" {
        cmd.arg("-w").arg("300x200");
    }

    // Command to execute in the new window
    let _ = cmd
        .arg(class_flag)
        .arg(class_name)
        .arg("-e")
        .arg(exe_path)
        .arg(time)
        .arg("--ghost-mode")
        .spawn()
        .expect("Failed to launch terminal. Is it installed?");
}

fn parse_duration(time_str: &str) -> u64 {
    let len = time_str.len();
    if len < 2 {
        return 0;
    } // Basic safety

    // Split the number and the unit (last character)
    let (num_part, unit) = time_str.split_at(len - 1);

    let number: u64 = num_part.parse().unwrap_or(0);

    match unit {
        "s" => number,
        "m" => number * 60,
        "h" => number * 60 * 60,
        _ => number, // Default to seconds if no unit provided
    }
}
