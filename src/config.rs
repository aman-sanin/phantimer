use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub terminal: Option<String>,
    pub presets: Option<BTreeMap<String, String>>,
    pub pomodoro: Option<PomodoroConfig>,
    pub colors: Option<ColorsConfig>,
    pub hyprland: Option<HyprlandConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PomodoroConfig {
    pub work: Option<String>,
    pub short_break: Option<String>,
    pub long_break: Option<String>,
    pub rounds: Option<u32>,
    pub long_break_interval: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ColorsConfig {
    pub timer_text: Option<String>,
    pub work_session: Option<String>,
    pub break_session: Option<String>,
    pub paused_text: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HyprlandConfig {
    pub rules: Option<Vec<String>>,
}

fn get_config_path() -> PathBuf {
    let base = std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/zero".to_string());
            PathBuf::from(home).join(".config")
        });
    base.join("phantimer").join("config.toml")
}

impl Config {
    pub fn load() -> Self {
        let path = get_config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    return config;
                }
            }
        }
        Config::default()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            terminal: Some("foot".to_string()),
            presets: Some({
                let mut m = BTreeMap::new();
                m.insert("Tea Break".to_string(), "25m".to_string());
                m.insert("Short Break".to_string(), "5m".to_string());
                m.insert("Long Break".to_string(), "15m".to_string());
                m.insert("Meeting".to_string(), "1h".to_string());
                m.insert("Standup".to_string(), "15m".to_string());
                m
            }),
            pomodoro: Some(PomodoroConfig {
                work: Some("25m".to_string()),
                short_break: Some("5m".to_string()),
                long_break: Some("15m".to_string()),
                rounds: Some(4),
                long_break_interval: Some(4),
            }),
            colors: Some(ColorsConfig {
                timer_text: Some("Cyan".to_string()),
                work_session: Some("Red".to_string()),
                break_session: Some("Green".to_string()),
                paused_text: Some("Yellow".to_string()),
            }),
            hyprland: Some(HyprlandConfig {
                rules: Some(vec![
                    "match:class ^(floating-timer)$, size 300 150".to_string(),
                    "match:class ^(floating-timer)$, move (monitor_w-310) 50".to_string(),
                    "match:class ^(floating-timer)$, float true".to_string(),
                    "match:class ^(floating-timer)$, pin true".to_string(),
                    "match:class ^(floating-timer)$, noborder true".to_string(),
                    "match:class ^(floating-timer)$, opacity 0.9 0.2".to_string(),
                ]),
            }),
        }
    }
}

pub fn parse_color(c: &str) -> ratatui::style::Color {
    match c.to_lowercase().as_str() {
        "black" => ratatui::style::Color::Black,
        "red" => ratatui::style::Color::Red,
        "green" => ratatui::style::Color::Green,
        "yellow" => ratatui::style::Color::Yellow,
        "blue" => ratatui::style::Color::Blue,
        "magenta" => ratatui::style::Color::Magenta,
        "cyan" => ratatui::style::Color::Cyan,
        "gray" => ratatui::style::Color::Gray,
        "darkgray" => ratatui::style::Color::DarkGray,
        "lightred" => ratatui::style::Color::LightRed,
        "lightgreen" => ratatui::style::Color::LightGreen,
        "lightyellow" => ratatui::style::Color::LightYellow,
        "lightblue" => ratatui::style::Color::LightBlue,
        "lightmagenta" => ratatui::style::Color::LightMagenta,
        "lightcyan" => ratatui::style::Color::LightCyan,
        "white" => ratatui::style::Color::White,
        _ => ratatui::style::Color::Cyan,
    }
}
