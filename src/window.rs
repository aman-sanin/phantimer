use std::process::Command;

pub fn detect_terminal(user_arg: Option<String>, config: &crate::config::Config) -> String {
    if let Some(t) = user_arg {
        return t;
    }
    if let Some(env_term) = std::env::var("TERMINAL").ok() {
        return env_term;
    }
    if let Some(ref config_term) = config.terminal {
        return config_term.clone();
    }
    "foot".to_string()
}

pub fn spawn_ghost_window(
    terminal: &str,
    time: Option<&str>,
    is_pomodoro: bool,
    work: Option<&str>,
    short_break: Option<&str>,
    long_break: Option<&str>,
    rounds: Option<u32>,
    interval: Option<u32>,
    config: &crate::config::Config,
) {
    let current_exe = std::env::current_exe().expect("Failed to get current executable path");
    let exe_path = current_exe.to_str().expect("Path contains invalid unicode");

    let (class_flag, class_name) = match terminal {
        "foot" => ("--app-id", "floating-timer"),
        _ => ("--class", "floating-timer"),
    };

    // Apply Hyprland Rules if on Linux
    if cfg!(target_os = "linux") {
        apply_hyprland_rules(class_name, config);
    }

    println!("Spawning {}...", terminal);

    let mut cmd = Command::new(terminal);

    // Foot specific sizing
    if terminal == "foot" {
        cmd.arg("-w").arg("300x150");
    }

    cmd.arg(class_flag).arg(class_name).arg("-e").arg(exe_path);
    if is_pomodoro {
        cmd.arg("--pomodoro");
        if let Some(w) = work {
            cmd.arg("--work").arg(w);
        }
        if let Some(s) = short_break {
            cmd.arg("--short-break").arg(s);
        }
        if let Some(l) = long_break {
            cmd.arg("--long-break").arg(l);
        }
        if let Some(r) = rounds {
            cmd.arg("--rounds").arg(r.to_string());
        }
        if let Some(i) = interval {
            cmd.arg("--interval").arg(i.to_string());
        }
    } else if let Some(t) = time {
        cmd.arg(t);
    }
    cmd.arg("--ghost-mode");

    let _ = cmd.spawn().expect("Failed to launch terminal");
}

fn apply_hyprland_rules(class_name: &str, config: &crate::config::Config) {
    let default_rules = vec![
        format!("match:class ^({})$, size 300 150", class_name),
        format!("match:class ^({})$, move (monitor_w-310) 50", class_name),
        format!("match:class ^({})$, float true", class_name),
        format!("match:class ^({})$, pin true", class_name),
        format!("match:class ^({})$, noborder true", class_name),
        // Interactive Transparency: 0.9 active, 0.4 inactive
        format!("match:class ^({})$, opacity 0.9 0.2", class_name),
    ];

    let rules = if let Some(ref hypr) = config.hyprland {
        if let Some(ref custom_rules) = hypr.rules {
            custom_rules.iter().map(|r| r.replace("floating-timer", class_name)).collect()
        } else {
            default_rules
        }
    } else {
        default_rules
    };

    for rule in rules {
        let _ = Command::new("hyprctl")
            .args(["keyword", "windowrule", &rule])
            .output();
    }
}
