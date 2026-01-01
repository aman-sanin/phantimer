use std::process::Command;

pub fn detect_terminal(user_arg: Option<String>) -> String {
    if let Some(t) = user_arg {
        return t;
    }
    if let Some(env_term) = std::env::var("TERMINAL").ok() {
        return env_term;
    }
    "foot".to_string()
}

pub fn spawn_ghost_window(terminal: &str, time: &str) {
    let current_exe = std::env::current_exe().expect("Failed to get current executable path");
    let exe_path = current_exe.to_str().expect("Path contains invalid unicode");

    let (class_flag, class_name) = match terminal {
        "foot" => ("--app-id", "floating-timer"),
        _ => ("--class", "floating-timer"),
    };

    // Apply Hyprland Rules if on Linux
    if cfg!(target_os = "linux") {
        apply_hyprland_rules(class_name);
    }

    println!("Spawning {}...", terminal);

    let mut cmd = Command::new(terminal);

    // Foot specific sizing
    if terminal == "foot" {
        cmd.arg("-w").arg("300x150");
    }

    let _ = cmd
        .args([class_flag, class_name, "-e", exe_path, time, "--ghost-mode"])
        .spawn()
        .expect("Failed to launch terminal");
}

fn apply_hyprland_rules(class_name: &str) {
    let rules = [
        format!("match:class ^({})$, size 300 150", class_name),
        format!("match:class ^({})$, move (monitor_w-310) 50", class_name),
        format!("match:class ^({})$, float true", class_name),
        format!("match:class ^({})$, pin true", class_name),
        format!("match:class ^({})$, noborder true", class_name),
        // Interactive Transparency: 0.9 active, 0.4 inactive
        format!("match:class ^({})$, opacity 0.9 0.2", class_name),
    ];

    for rule in rules {
        let _ = Command::new("hyprctl")
            .args(["keyword", "windowrule", &rule])
            .output();
    }
}
