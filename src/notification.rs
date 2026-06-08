use std::process::Command;

/// Sends a non-blocking system desktop notification using `notify-send`.
pub fn send(title: &str, body: &str) {
    let _ = Command::new("notify-send")
        .args([
            "-a", "Phantimer",
            "-i", "alarm-symbolic",
            title,
            body,
        ])
        .spawn();
}
