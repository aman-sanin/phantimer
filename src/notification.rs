use std::process::Command;

/// Sends a non-blocking system desktop notification using `notify-send`
/// and optionally plays a completion audio alert if `sound` is true.
pub fn send(title: &str, body: &str, sound: bool) {
    let _ = Command::new("notify-send")
        .args([
            "-a", "Phantimer",
            "-i", "alarm-symbolic",
            title,
            body,
        ])
        .spawn();

    if sound {
        play_sound();
    }
}

/// Plays a non-blocking completion audio cue using system sound utilities.
pub fn play_sound() {
    // 1. Try canberra-gtk-play (standard event sound player)
    let res = Command::new("canberra-gtk-play")
        .args(["-i", "complete", "-d", "Phantimer"])
        .spawn();

    if res.is_err() {
        // 2. Fallback to paplay or pw-play with freedesktop sound files
        let sound_files = [
            "/usr/share/sounds/freedesktop/stereo/complete.oga",
            "/usr/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga",
            "/usr/share/sounds/freedesktop/stereo/bell.oga",
        ];
        for file in sound_files {
            if std::path::Path::new(file).exists() {
                if Command::new("paplay").arg(file).spawn().is_ok() {
                    return;
                }
                if Command::new("pw-play").arg(file).spawn().is_ok() {
                    return;
                }
            }
        }
    }
}
