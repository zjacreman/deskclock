use serde::Deserialize;
use std::path::PathBuf;

pub(crate) fn color_from_str(s: &str) -> ratatui::style::Color {
    match s {
        "white" | "White" | "WHITE" => ratatui::style::Color::White,
        "black" | "Black" | "BLACK" => ratatui::style::Color::Black,
        "red" | "Red" | "RED" => ratatui::style::Color::Red,
        "green" | "Green" | "GREEN" => ratatui::style::Color::Green,
        "yellow" | "Yellow" | "YELLOW" => ratatui::style::Color::Yellow,
        "blue" | "Blue" | "BLUE" => ratatui::style::Color::Blue,
        "magenta" | "Magenta" | "MAGENTA" => ratatui::style::Color::Magenta,
        "pink" | "Pink" | "PINK" => ratatui::style::Color::Magenta,
        "cyan" | "Cyan" | "CYAN" => ratatui::style::Color::Cyan,
        "gray" | "Gray" | "GRAY" | "grey" | "Grey" | "GREY" => ratatui::style::Color::Gray,
        "dark_gray" | "darkgrey" | "DarkGray" | "DarkGrey" | "DARK_GRAY" | "DARKGREY" => {
            ratatui::style::Color::DarkGray
        }
        "light_red" | "lightred" | "LightRed" | "LIGHT_RED" | "LIGHTRED" => {
            ratatui::style::Color::LightRed
        }
        "light_green" | "lightgreen" | "LightGreen" | "LIGHT_GREEN" | "LIGHTGREEN" => {
            ratatui::style::Color::LightGreen
        }
        "light_yellow" | "lightyellow" | "LightYellow" | "LIGHT_YELLOW" | "LIGHTYELLOW" => {
            ratatui::style::Color::LightYellow
        }
        "light_blue" | "lightblue" | "LightBlue" | "LIGHT_BLUE" | "LIGHTBLUE" => {
            ratatui::style::Color::LightBlue
        }
        "light_magenta" | "lightmagenta" | "LightMagenta" | "light_pink" | "lightpink"
        | "LightPink" | "LIGHT_MAGENTA" | "LIGHTPINK" => ratatui::style::Color::LightMagenta,
        "light_cyan" | "lightcyan" | "LightCyan" | "LIGHT_CYAN" | "LIGHTCYAN" => {
            ratatui::style::Color::LightCyan
        }
        s if s.starts_with('#') && s.len() == 7 => {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&s[1..3], 16),
                u8::from_str_radix(&s[3..5], 16),
                u8::from_str_radix(&s[5..7], 16),
            ) {
                return ratatui::style::Color::Rgb(r, g, b);
            }
            eprintln!("deskclock: warning: invalid hex color '{}', defaulting to White", s);
            ratatui::style::Color::White
        }
        s if s.starts_with("rgb(") && s.ends_with(')') => {
            let inner = &s[4..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() == 3 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    parts[0].trim().parse::<u8>(),
                    parts[1].trim().parse::<u8>(),
                    parts[2].trim().parse::<u8>(),
                ) {
                    return ratatui::style::Color::Rgb(r, g, b);
                }
            }
            eprintln!("deskclock: warning: invalid rgb color '{}', defaulting to White", s);
            ratatui::style::Color::White
        }
        _ => {
            eprintln!("deskclock: warning: unrecognized color '{}', defaulting to White", s);
            ratatui::style::Color::White
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefaultMode {
    Time,
    Countdown,
    Stopwatch,
}

fn mode_from_str(s: &str) -> DefaultMode {
    match s.to_lowercase().as_str() {
        "time" => DefaultMode::Time,
        "countdown" => DefaultMode::Countdown,
        "stopwatch" => DefaultMode::Stopwatch,
        _ => DefaultMode::Time,
    }
}

#[derive(Debug, Clone)]
pub struct ColorConfig {
    pub time_color: ratatui::style::Color,
    pub date_color: ratatui::style::Color,
    pub countdown_running_color: ratatui::style::Color,
    pub countdown_idle_color: ratatui::style::Color,
    pub stopwatch_running_color: ratatui::style::Color,
    pub stopwatch_idle_color: ratatui::style::Color,
    pub menu_color: ratatui::style::Color,
    pub alert_color: ratatui::style::Color,
    pub stopwatch_lap_color: ratatui::style::Color,
}

impl ColorConfig {
    fn default_colors() -> Self {
        Self {
            time_color: color_from_str("White"),
            date_color: color_from_str("Yellow"),
            countdown_running_color: color_from_str("Cyan"),
            countdown_idle_color: color_from_str("White"),
            stopwatch_running_color: color_from_str("Magenta"),
            stopwatch_idle_color: color_from_str("White"),
            menu_color: color_from_str("DarkGray"),
            alert_color: color_from_str("Red"),
            stopwatch_lap_color: color_from_str("Blue"),
        }
    }
}

#[derive(Debug)]
pub struct AppConfig {
    pub colors: ColorConfig,
    pub countdown_default_seconds: u64,
    pub use_24h_format: bool,
    pub default_mode: DefaultMode,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            colors: ColorConfig::default_colors(),
            countdown_default_seconds: 25 * 60,
            use_24h_format: false,
            default_mode: DefaultMode::Time,
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let paths = collect_config_paths();

        for path in &paths {
            let full_path = match path.to_str() {
                Some(s) => s.to_owned(),
                None => continue,
            };

            if let Ok(content) = std::fs::read_to_string(&full_path) {
                match load_from_content(&content) {
                    Ok(cfg) => {
                        println!("deskclock: loaded config from {}", path.display());
                        return cfg;
                    }
                    Err(e) => {
                        println!("deskclock: failed to parse config at {}: {}", path.display(), e);
                    }
                }
            } else {
                println!("deskclock: checking {}", path.display());
            }
        }

        println!("deskclock: using default configuration");
        Self::default()
    }
}

// ============================================================
// Deserialization types
// ============================================================

#[derive(Deserialize)]
#[serde(default = "default_raw_config")]
struct RawConfig {
    time_color: String,
    date_color: String,
    countdown_running_color: String,
    countdown_idle_color: String,
    stopwatch_running_color: String,
    stopwatch_idle_color: String,
    menu_color: String,
    alert_color: String,
    stopwatch_lap_color: String,
    countdown_default_seconds: u64,
    use_24h_format: bool,
    default_mode: Option<String>,
}

// Function that serde calls when no config file is found at all
// (the entire struct is missing from the parsed TOML).
fn default_raw_config() -> RawConfig {
    RawConfig {
        time_color: "White".into(),
        date_color: "Yellow".into(),
        countdown_running_color: "Cyan".into(),
        countdown_idle_color: "White".into(),
        stopwatch_running_color: "Magenta".into(),
        stopwatch_idle_color: "White".into(),
        menu_color: "DarkGray".into(),
        alert_color: "Red".into(),
        stopwatch_lap_color: "Blue".into(),
        countdown_default_seconds: 25 * 60,
        use_24h_format: false,
        default_mode: None,
    }
}

fn load_from_content(content: &str) -> Result<AppConfig, String> {
    let raw: RawConfig = toml::from_str(content)
        .map_err(|e: toml::de::Error| format!("Failed to parse config: {}", e))?;

    let default_mode = raw
        .default_mode
        .as_deref()
        .map(mode_from_str)
        .unwrap_or(DefaultMode::Time);

    Ok(AppConfig {
        colors: ColorConfig {
            time_color: color_from_str(&raw.time_color),
            date_color: color_from_str(&raw.date_color),
            countdown_running_color: color_from_str(&raw.countdown_running_color),
            countdown_idle_color: color_from_str(&raw.countdown_idle_color),
            stopwatch_running_color: color_from_str(&raw.stopwatch_running_color),
            stopwatch_idle_color: color_from_str(&raw.stopwatch_idle_color),
            menu_color: color_from_str(&raw.menu_color),
            alert_color: color_from_str(&raw.alert_color),
            stopwatch_lap_color: color_from_str(&raw.stopwatch_lap_color),
        },
        countdown_default_seconds: raw.countdown_default_seconds,
        use_24h_format: raw.use_24h_format,
        default_mode,
    })
}

fn collect_config_paths() -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from("config.toml")];

    // ~/.config/deskclock/config.toml
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".config").join("deskclock").join("config.toml"));
    }

    // Platform-standard config (XDG on Unix, AppData/roaming on Windows)
    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("deskclock").join("config.toml"));
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_str_named_colors() {
        assert_eq!(color_from_str("Red"), ratatui::style::Color::Red);
        assert_eq!(color_from_str("Cyan"), ratatui::style::Color::Cyan);
        assert_eq!(color_from_str("White"), ratatui::style::Color::White);
        assert_eq!(color_from_str("Yellow"), ratatui::style::Color::Yellow);
        assert_eq!(color_from_str("Magenta"), ratatui::style::Color::Magenta);
        assert_eq!(color_from_str("DarkGray"), ratatui::style::Color::DarkGray);
    }

    #[test]
    fn test_color_from_str_hex() {
        assert_eq!(
            color_from_str("#FF5733"),
            ratatui::style::Color::Rgb(255, 87, 51)
        );
    }

    #[test]
    fn test_color_from_str_rgb() {
        assert_eq!(
            color_from_str("rgb(10, 20, 30)"),
            ratatui::style::Color::Rgb(10, 20, 30)
        );
    }

    #[test]
    fn test_default_config_values() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.countdown_default_seconds, 25 * 60);
        assert!(!cfg.use_24h_format);
        assert_eq!(cfg.default_mode, DefaultMode::Time);
        assert_eq!(cfg.colors.stopwatch_lap_color, ratatui::style::Color::Blue);
    }

    #[test]
    fn test_load_empty_content_returns_defaults() {
        let cfg = load_from_content("").expect("should parse empty toml");
        assert_eq!(cfg.countdown_default_seconds, 25 * 60);
        assert!(!cfg.use_24h_format);
        assert_eq!(cfg.default_mode, DefaultMode::Time);
    }

    #[test]
    fn test_load_custom_values() {
        let toml_str = r#"
countdown_default_seconds = 300
use_24h_format = true
default_mode = "countdown"
time_color = "Red"
stopwatch_lap_color = "Cyan"
"#;
        let cfg = load_from_content(toml_str).expect("should parse toml");
        assert_eq!(cfg.colors.time_color, ratatui::style::Color::Red);
        assert_eq!(cfg.colors.date_color, ratatui::style::Color::Yellow);
        assert_eq!(cfg.countdown_default_seconds, 300);
        assert!(cfg.use_24h_format);
        assert_eq!(cfg.default_mode, DefaultMode::Countdown);
        assert_eq!(cfg.colors.stopwatch_lap_color, ratatui::style::Color::Cyan);
    }

    #[test]
    fn test_color_from_str_colors() {
        assert_eq!(
            color_from_str("Red"),
            ratatui::style::Color::Red
        );
        assert_eq!(
            color_from_str("#FF5733"),
            ratatui::style::Color::Rgb(255, 87, 51)
        );
        assert_eq!(
            color_from_str("rgb(10, 20, 30)"),
            ratatui::style::Color::Rgb(10, 20, 30)
        );
        assert_eq!(
            color_from_str("Cyan"),
            ratatui::style::Color::Cyan
        );
    }
}
