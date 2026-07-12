use serde::Deserialize;
use std::path::PathBuf;

/// Error returned when a color string cannot be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ColorParseError {
    pub input: String,
}

impl std::fmt::Display for ColorParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid or unrecognized color '{}'", self.input)
    }
}

impl std::error::Error for ColorParseError {}

/// Parse a color specification into a `Color`.
///
/// Accepted forms: named colors (case-insensitive, with several spelling
/// variants), hex `#RRGGBB`, and `rgb(R, G, B)`.
///
/// This is the fallible core; [`color_from_str`] wraps it with a warning +
/// fallback to `White`.
pub(crate) fn parse_color(s: &str) -> Result<ratatui::style::Color, ColorParseError> {
    use ratatui::style::Color;
    Ok(match s {
        "white" | "White" | "WHITE" => Color::White,
        "black" | "Black" | "BLACK" => Color::Black,
        "red" | "Red" | "RED" => Color::Red,
        "green" | "Green" | "GREEN" => Color::Green,
        "yellow" | "Yellow" | "YELLOW" => Color::Yellow,
        "blue" | "Blue" | "BLUE" => Color::Blue,
        "magenta" | "Magenta" | "MAGENTA" => Color::Magenta,
        "pink" | "Pink" | "PINK" => Color::Magenta,
        "cyan" | "Cyan" | "CYAN" => Color::Cyan,
        "gray" | "Gray" | "GRAY" | "grey" | "Grey" | "GREY" => Color::Gray,
        "dark_gray" | "darkgrey" | "DarkGray" | "DarkGrey" | "DARK_GRAY" | "DARKGREY" => {
            Color::DarkGray
        }
        "light_red" | "lightred" | "LightRed" | "LIGHT_RED" | "LIGHTRED" => Color::LightRed,
        "light_green" | "lightgreen" | "LightGreen" | "LIGHT_GREEN" | "LIGHTGREEN" => {
            Color::LightGreen
        }
        "light_yellow" | "lightyellow" | "LightYellow" | "LIGHT_YELLOW" | "LIGHTYELLOW" => {
            Color::LightYellow
        }
        "light_blue" | "lightblue" | "LightBlue" | "LIGHT_BLUE" | "LIGHTBLUE" => Color::LightBlue,
        "light_magenta" | "lightmagenta" | "LightMagenta" | "light_pink" | "lightpink"
        | "LightPink" | "LIGHT_MAGENTA" | "LIGHTPINK" => Color::LightMagenta,
        "light_cyan" | "lightcyan" | "LightCyan" | "LIGHT_CYAN" | "LIGHTCYAN" => Color::LightCyan,
        s if s.starts_with('#') && s.len() == 7 => {
            let r = u8::from_str_radix(&s[1..3], 16)
                .map_err(|_| ColorParseError { input: s.into() })?;
            let g = u8::from_str_radix(&s[3..5], 16)
                .map_err(|_| ColorParseError { input: s.into() })?;
            let b = u8::from_str_radix(&s[5..7], 16)
                .map_err(|_| ColorParseError { input: s.into() })?;
            Color::Rgb(r, g, b)
        }
        s if s.starts_with("rgb(") && s.ends_with(')') => {
            let inner = &s[4..s.len() - 1];
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() != 3 {
                return Err(ColorParseError { input: s.into() });
            }
            let r = parts[0]
                .trim()
                .parse::<u8>()
                .map_err(|_| ColorParseError { input: s.into() })?;
            let g = parts[1]
                .trim()
                .parse::<u8>()
                .map_err(|_| ColorParseError { input: s.into() })?;
            let b = parts[2]
                .trim()
                .parse::<u8>()
                .map_err(|_| ColorParseError { input: s.into() })?;
            Color::Rgb(r, g, b)
        }
        other => return Err(ColorParseError { input: other.into() }),
    })
}

/// Parse a color, falling back to `White` (with a stderr warning) on failure.
pub(crate) fn color_from_str(s: &str) -> ratatui::style::Color {
    match parse_color(s) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("deskclock: warning: {}, defaulting to White", e);
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

/// Color theme for the application. Defaults are derived from [`default_raw_config`]
/// (see [`ColorConfig::default`]) so there is a single source of truth shared with
/// the TOML deserializer.
impl ColorConfig {
    fn from_raw(r: &RawConfig) -> Self {
        Self {
            time_color: color_from_str(&r.time_color),
            date_color: color_from_str(&r.date_color),
            countdown_running_color: color_from_str(&r.countdown_running_color),
            countdown_idle_color: color_from_str(&r.countdown_idle_color),
            stopwatch_running_color: color_from_str(&r.stopwatch_running_color),
            stopwatch_idle_color: color_from_str(&r.stopwatch_idle_color),
            menu_color: color_from_str(&r.menu_color),
            alert_color: color_from_str(&r.alert_color),
            stopwatch_lap_color: color_from_str(&r.stopwatch_lap_color),
        }
    }
}

impl Default for ColorConfig {
    fn default() -> Self {
        ColorConfig::from_raw(&default_raw_config())
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
        let r = default_raw_config();
        Self {
            colors: ColorConfig::default(),
            countdown_default_seconds: r.countdown_default_seconds,
            use_24h_format: r.use_24h_format,
            default_mode: r
                .default_mode
                .as_deref()
                .map(mode_from_str)
                .unwrap_or(DefaultMode::Time),
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

// Single source of truth for all configuration defaults. serde calls this for
// any field missing from the parsed TOML, and `ColorConfig::default` /
// `AppConfig::default` derive from it so the three sources can never drift.
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
        colors: ColorConfig::from_raw(&raw),
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

    // ============================================================
    // Fallible color parsing (parse_color)
    // ============================================================

    #[test]
    fn test_parse_color_ok_named() {
        assert_eq!(parse_color("Red"), Ok(ratatui::style::Color::Red));
        assert_eq!(parse_color("pink"), Ok(ratatui::style::Color::Magenta));
    }

    #[test]
    fn test_parse_color_ok_hex_and_rgb() {
        assert_eq!(
            parse_color("#00ff00"),
            Ok(ratatui::style::Color::Rgb(0, 255, 0))
        );
        assert_eq!(
            parse_color("rgb(1, 2, 3)"),
            Ok(ratatui::style::Color::Rgb(1, 2, 3))
        );
    }

    #[test]
    fn test_parse_color_err_unrecognized() {
        assert!(parse_color("notacolor").is_err());
    }

    #[test]
    fn test_parse_color_err_bad_hex() {
        // Wrong length for hex.
        assert!(parse_color("#1234567").is_err());
        // Non-hex digits.
        assert!(parse_color("#zzzzzz").is_err());
    }

    #[test]
    fn test_parse_color_err_bad_rgb() {
        // Too few components.
        assert!(parse_color("rgb(1, 2)").is_err());
        // Out-of-range value.
        assert!(parse_color("rgb(300, 1, 1)").is_err());
    }

    #[test]
    fn test_color_from_str_falls_back_to_white_on_error() {
        assert_eq!(
            color_from_str("notacolor"),
            ratatui::style::Color::White
        );
    }

    #[test]
    fn test_color_parse_error_displays_input() {
        let e = parse_color("bogus").unwrap_err();
        let s = format!("{}", e);
        assert!(s.contains("bogus"), "error message should mention the input");
    }

    // ============================================================
    // Default source consolidation
    // ============================================================

    #[test]
    fn test_default_raw_config_is_single_source_for_defaults() {
        // ColorConfig::default() and AppConfig::default() must both be derived
        // from default_raw_config(), so they stay in sync.
        let raw = default_raw_config();
        let colors = ColorConfig::default();
        assert_eq!(colors.time_color, color_from_str(&raw.time_color));
        assert_eq!(colors.date_color, color_from_str(&raw.date_color));
        assert_eq!(
            colors.countdown_running_color,
            color_from_str(&raw.countdown_running_color)
        );
        assert_eq!(
            colors.stopwatch_lap_color,
            color_from_str(&raw.stopwatch_lap_color)
        );

        let cfg = AppConfig::default();
        assert_eq!(cfg.countdown_default_seconds, raw.countdown_default_seconds);
        assert_eq!(cfg.use_24h_format, raw.use_24h_format);
        assert_eq!(
            cfg.default_mode,
            raw.default_mode
                .as_deref()
                .map(mode_from_str)
                .unwrap_or(DefaultMode::Time)
        );
    }

    #[test]
    fn test_load_missing_fields_use_defaults() {
        // Providing only one field should leave the rest at default_raw_config values.
        let cfg = load_from_content("countdown_default_seconds = 99\n").expect("ok");
        assert_eq!(cfg.countdown_default_seconds, 99);
        // The rest come from defaults.
        assert!(!cfg.use_24h_format);
        assert_eq!(cfg.default_mode, DefaultMode::Time);
        assert_eq!(cfg.colors.time_color, ratatui::style::Color::White);
    }
}
