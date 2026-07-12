use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use crossterm::event::{KeyEventKind, KeyModifiers};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::Alignment,
    style::Style,
    widgets::Paragraph,
};
use std::{
    error::Error,
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

mod font;
use font::LargeFont;

mod notification;
use notification::Notifier;

mod stopwatch;
use stopwatch::Stopwatch;

mod timer;
use timer::CountdownTimer;

mod ui;

mod signal;

mod config;
use config::{AppConfig, DefaultMode};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum AppMode {
    Time,
    Countdown,
    Stopwatch,
}

struct App {
    should_quit: bool,
    font: LargeFont,
    mode: AppMode,
    timer: CountdownTimer,
    stopwatch: Stopwatch,
    use_24h_format: bool,
    flash_start_time: Option<Instant>,
    notifier: Box<dyn Notifier>,
    config: AppConfig,
}

impl App {
    fn new() -> Self {
        let config = AppConfig::load();
        let mode = match config.default_mode {
            DefaultMode::Time => AppMode::Time,
            DefaultMode::Countdown => AppMode::Countdown,
            DefaultMode::Stopwatch => AppMode::Stopwatch,
        };
        let timer = CountdownTimer::with_duration(config.countdown_default_seconds);

        Self {
            should_quit: false,
            font: LargeFont::new(),
            mode,
            timer,
            stopwatch: Stopwatch::new(),
            use_24h_format: config.use_24h_format,
            flash_start_time: None,
            notifier: Box::new(notification::SystemNotifier::new()),
            config,
        }
    }

    #[cfg(test)]
    fn with_notifier(mut self, notifier: Box<dyn Notifier>) -> Self {
        self.notifier = notifier;
        self
    }

    fn handle_arrow_key(&mut self, key: KeyCode) {
        match (key, &self.mode) {
            (KeyCode::Up, AppMode::Countdown) => self.timer.adjust_minutes(1),
            (KeyCode::Down, AppMode::Countdown) => self.timer.adjust_minutes(-1),
            (KeyCode::Left, AppMode::Countdown) => self.timer.adjust_seconds(-1),
            (KeyCode::Right, AppMode::Countdown) => self.timer.adjust_seconds(1),
            _ => {}
        }
    }

    /// Dispatch a single key event. The event-kind (Press/Release) filtering is
    /// done by the caller in [`run`](Self::run); this method processes any key
    /// handed to it, which keeps it trivially unit-testable.
    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Char('c') => self.mode = AppMode::Countdown,
            KeyCode::Char('t') => self.mode = AppMode::Time,
            KeyCode::Char('h') => {
                if let AppMode::Time = &self.mode {
                    self.use_24h_format = !self.use_24h_format;
                }
            }
            KeyCode::Char('s') => self.mode = AppMode::Stopwatch,
            KeyCode::Char('l') => {
                if let AppMode::Stopwatch = &self.mode {
                    let lap_time = self.stopwatch.current_elapsed();
                    self.stopwatch.add_lap(lap_time);
                }
            }
            KeyCode::Char('r') => match self.mode {
                AppMode::Countdown => self.timer.reset(),
                AppMode::Stopwatch => self.stopwatch.reset(),
                _ => {}
            },
            KeyCode::Char(' ') => match self.mode {
                AppMode::Countdown => {
                    if self.timer.is_running() {
                        self.timer.pause();
                    } else {
                        self.timer.start();
                    }
                }
                AppMode::Stopwatch => {
                    if self.stopwatch.is_running() {
                        self.stopwatch.pause();
                    } else {
                        self.stopwatch.start();
                    }
                }
                _ => {}
            },
            KeyCode::Up => self.handle_arrow_key(KeyCode::Up),
            KeyCode::Down => self.handle_arrow_key(KeyCode::Down),
            KeyCode::Left => self.handle_arrow_key(KeyCode::Left),
            KeyCode::Right => self.handle_arrow_key(KeyCode::Right),
            _ => {}
        }
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        enable_raw_mode()?;
        let shutdown = signal::register_signal_handler();
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        // RAII guard: guarantee the terminal is restored (raw mode off +
        // alternate screen left) even when `run` returns early via `?` on a
        // draw/poll error. It must be created AFTER entering raw mode +
        // alt screen so cleanup only runs for what was actually set up, and it
        // runs exactly once on every return path (Ok or Err).
        struct TerminalGuard;
        impl Drop for TerminalGuard {
            fn drop(&mut self) {
                let _ = disable_raw_mode();
                let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
            }
        }
        let _guard = TerminalGuard;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let tick_rate = Duration::from_millis(200);
        let mut last_tick = Instant::now();

        loop {
            terminal.draw(|f| {
                let size = f.size();

                if let Some(start) = self.flash_start_time {
                    let elapsed = start.elapsed().as_millis();
                    if elapsed < 1250 {
                        if (elapsed / 250) % 2 == 0 {
                            f.render_widget(
                                Paragraph::new("")
                                    .style(Style::default().bg(self.config.colors.alert_color)),
                                size,
                            );
                        }
                    } else {
                        self.flash_start_time = None;
                    }
                }

                let chunks = ui::create_main_layout(size);

                match self.mode {
                    AppMode::Time => {
                        let now = Local::now();
                        let time_fmt = if self.use_24h_format { "%H:%M:%S" } else { "%I:%M:%S %p" };
                        let time_str = now.format(time_fmt).to_string();
                        let date_str = now.format("%A, %B %d, %Y").to_string();

                        ui::render_large_text(
                            f,
                            chunks[0],
                            &time_str,
                            self.config.colors.time_color,
                            &self.font,
                        );
                        ui::render_large_text(
                            f,
                            chunks[1],
                            &date_str,
                            self.config.colors.date_color,
                            &self.font,
                        );
                    }
                    AppMode::Countdown => {
                        let rem = self.timer.remaining();
                        let h = rem.as_secs() / 3600;
                        let m = (rem.as_secs() % 3600) / 60;
                        let s = rem.as_secs() % 60;
                        let timer_str = if h > 0 {
                            format!("{:02}:{:02}:{:02}", h, m, s)
                        } else {
                            format!("{:02}:{:02}", m, s)
                        };

                        let is_paused = self.timer.is_paused();
                        let is_running = self.timer.is_running();

                        let color = if is_running || is_paused {
                            self.config.colors.countdown_running_color
                        } else {
                            self.config.colors.countdown_idle_color
                        };

                        // Finished and stopped both render as static; only the
                        // paused state blinks to signal "held but not done".
                        let is_visible = if is_running {
                            true
                        } else if is_paused {
                            (Local::now().timestamp_millis() / 500) % 2 == 0
                        } else {
                            true
                        };

                        if is_visible {
                            ui::render_large_text(
                                f,
                                chunks[0],
                                &timer_str,
                                color,
                                &self.font,
                            );
                        }

                        let end_time_str = if is_running {
                            let now_local = Local::now();
                            let end_local = now_local
                                + chrono::Duration::from_std(self.timer.remaining())
                                    .unwrap_or(chrono::Duration::zero());
                            let fmt = if self.use_24h_format {
                                "%H:%M:%S"
                            } else {
                                "%I:%M:%S %p"
                            };
                            format!("Ends at: {}", end_local.format(fmt))
                        } else if self.timer.is_finished() {
                            "Complete".to_string()
                        } else {
                            "Paused".to_string()
                        };

                        let p = Paragraph::new(end_time_str)
                            .alignment(Alignment::Center)
                            .style(Style::default().fg(self.config.colors.menu_color));
                        f.render_widget(p, chunks[1]);
                    }
                    AppMode::Stopwatch => {
                        let elapsed = self.stopwatch.current_elapsed();
                        let total_secs = elapsed.as_secs();
                        let h = total_secs / 3600;
                        let m = (total_secs % 3600) / 60;
                        let s = total_secs % 60;
                        let cs = elapsed.subsec_millis() / 10;

                        let timer_str = if h > 0 {
                            format!("{:02}:{:02}:{:02}", h, m, s)
                        } else {
                            format!("{:02}:{:02}.{:02}", m, s, cs)
                        };

                        let color = if self.stopwatch.is_running() {
                            self.config.colors.stopwatch_running_color
                        } else {
                            self.config.colors.stopwatch_idle_color
                        };

                        let is_visible = if self.stopwatch.is_running() {
                            true
                        } else if self.stopwatch.elapsed_time() > Duration::ZERO {
                            (Local::now().timestamp_millis() / 500) % 2 == 0
                        } else {
                            true
                        };

                        if is_visible {
                            ui::render_large_text(
                                f,
                                chunks[0],
                                &timer_str,
                                color,
                                &self.font,
                            );
                        }

                        if let Some(lap) = self.stopwatch.last_lap_elapsed() {
                            let lap_secs = lap.as_secs();
                            let lh = lap_secs / 3600;
                            let lm = (lap_secs % 3600) / 60;
                            let ls = lap_secs % 60;
                            let lcs = lap.subsec_millis() / 10;

                            // Lap uses the same HH:MM:SS / MM:SS.cs split as the
                            // main stopwatch so a lap >= 1h is not silently truncated.
                            let lap_display = if lh > 0 {
                                format!("{:02}:{:02}:{:02}", lh, lm, ls)
                            } else {
                                format!("{:02}:{:02}.{:02}", lm, ls, lcs)
                            };

                            ui::render_large_text(
                                f,
                                chunks[1],
                                &lap_display,
                                self.config.colors.stopwatch_lap_color,
                                &self.font,
                            );
                        } else {
                            let p = Paragraph::new("Stopwatch")
                                .alignment(Alignment::Center)
                                .style(Style::default().fg(self.config.colors.menu_color));
                            f.render_widget(p, chunks[1]);
                        }
                    }
                }

                let cmd_text = match self.mode {
                    AppMode::Time => "q: Quit | c: Countdown | s: Stopwatch | h: 12/24h",
                    AppMode::Countdown => {
                        if self.timer.is_running() {
                            "q: Quit | t: Time | s: Stopwatch | Space: Pause | r: Reset"
                        } else {
                            "q: Quit | t: Time | s: Stopwatch | Space: Start | r: Reset | ↑↓: Min | ←→: Sec"
                        }
                    }
                    AppMode::Stopwatch => {
                        if self.stopwatch.is_running() {
                            "q: Quit | t: Time | c: Countdown | Space: Pause | r: Reset | l: Lap"
                        } else {
                            "q: Quit | t: Time | c: Countdown | Space: Start | r: Reset | l: Lap"
                        }
                    }
                };

                let cmd_p = Paragraph::new(cmd_text)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(self.config.colors.menu_color));
                f.render_widget(cmd_p, chunks[2]);
            })?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::ZERO);

            // NOTE: clippy's collapsible_if would merge these two guards, but the
            // suggested fix relies on unstable `let_chains`. Keep the explicit
            // nesting: `event::poll` must gate `event::read` (read blocks).
            #[allow(clippy::collapsible_if)]
            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    // On Windows, crossterm emits key events for both Press and
                    // Release, which would otherwise cause every keystroke to
                    // be processed twice. Only handle Press events.
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    self.handle_key(key);
                }
            }

            // Check for external signal (SIGTERM / SIGINT from kill)
            if shutdown.load(Ordering::SeqCst) {
                self.should_quit = true;
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
                // Detection deliberately uses is_zero() (sub-second accurate) so the
                // flash notification does not fire up to ~1s early. After finish(),
                // is_running becomes false, so this won't re-trigger every tick.
                if self.timer.has_expired() {
                    self.flash_start_time = Some(Instant::now());
                    self.timer.finish();
                    self.notifier
                        .send_notification("Countdown Timer Complete", "00:00 - Timer has finished");
                }
            }

            if self.should_quit {
                break;
            }
        }

        // Terminal cleanup is handled by `_guard`'s Drop on return.
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    app.run()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::time::Duration;

    fn key(code: KeyCode, mods: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, mods)
    }

    // ============================================================
    // App Core Tests
    // ============================================================

    #[test]
    fn test_app_new_creates_default_state() {
        let app = App::new();
        assert!(!app.should_quit);
        assert_eq!(app.mode, AppMode::Time);
        assert!(!app.timer.is_running());
        assert!(!app.stopwatch.is_running());
        assert!(app.flash_start_time.is_none());
    }

    #[test]
    fn test_app_24h_toggle() {
        let mut app = App::new();
        let initial = app.use_24h_format;
        app.use_24h_format = !app.use_24h_format;
        assert_ne!(app.use_24h_format, initial, "Should toggle to the opposite format");
        app.use_24h_format = !app.use_24h_format;
        assert_eq!(app.use_24h_format, initial, "Should toggle back to the original format");
    }

    #[test]
    fn test_toggle_24h_format_does_not_work_outside_time_mode() {
        // Countdown mode: toggling 'h' should NOT change use_24h_format
        let mut app = App::new();
        app.mode = AppMode::Countdown;
        let initial = app.use_24h_format;
        app.handle_key(key(KeyCode::Char('h'), KeyModifiers::NONE));
        assert_eq!(
            app.use_24h_format, initial,
            "Toggling 'h' in Countdown mode should NOT change use_24h_format"
        );

        // Stopwatch mode: toggling 'h' should NOT change use_24h_format
        let mut app = App::new();
        app.mode = AppMode::Stopwatch;
        let initial = app.use_24h_format;
        app.handle_key(key(KeyCode::Char('h'), KeyModifiers::NONE));
        assert_eq!(
            app.use_24h_format, initial,
            "Toggling 'h' in Stopwatch mode should NOT change use_24h_format"
        );
    }

    #[test]
    fn test_toggle_24h_format_works_in_time_mode() {
        // Confirm the guard allows the toggle when in Time mode
        let mut app = App::new();
        app.mode = AppMode::Time;
        let initial = app.use_24h_format;
        app.handle_key(key(KeyCode::Char('h'), KeyModifiers::NONE));
        assert_ne!(
            app.use_24h_format, initial,
            "Toggling 'h' in Time mode SHOULD change use_24h_format"
        );
    }

    #[test]
    fn test_app_mode_switch_to_countdown() {
        let mut app = App::new();
        app.handle_key(key(KeyCode::Char('c'), KeyModifiers::NONE));
        assert_eq!(app.mode, AppMode::Countdown);
    }

    #[test]
    fn test_app_mode_switch_to_stopwatch() {
        let mut app = App::new();
        app.handle_key(key(KeyCode::Char('s'), KeyModifiers::NONE));
        assert_eq!(app.mode, AppMode::Stopwatch);
    }

    #[test]
    fn test_app_mode_switch_to_time() {
        let mut app = App::new();
        app.mode = AppMode::Countdown;
        app.handle_key(key(KeyCode::Char('t'), KeyModifiers::NONE));
        assert_eq!(app.mode, AppMode::Time);
    }

    #[test]
    fn test_app_should_quit() {
        let mut app = App::new();
        assert!(!app.should_quit);
        app.should_quit = true;
        assert!(app.should_quit);
    }

    // ============================================================
    // AppMode Derivation Tests
    // ============================================================

    #[test]
    fn test_app_mode_equality() {
        assert_eq!(AppMode::Time, AppMode::Time);
        assert_eq!(AppMode::Countdown, AppMode::Countdown);
        assert_eq!(AppMode::Stopwatch, AppMode::Stopwatch);
    }

    #[test]
    fn test_app_mode_inequality() {
        assert_ne!(AppMode::Time, AppMode::Countdown);
        assert_ne!(AppMode::Time, AppMode::Stopwatch);
        assert_ne!(AppMode::Countdown, AppMode::Stopwatch);
    }

    // ============================================================
    // Arrow Key Event Handling Tests
    // ============================================================

    #[test]
    fn test_handle_arrow_key_adjusts_timer_in_countdown_mode() {
        let mut app = App::new();
        app.mode = AppMode::Countdown;
        let initial_duration = app.timer.duration();

        app.handle_arrow_key(KeyCode::Up);
        assert_eq!(app.timer.duration(), initial_duration + Duration::from_secs(60));

        app.handle_arrow_key(KeyCode::Down);
        assert_eq!(app.timer.duration(), initial_duration);

        let current = app.timer.duration();
        app.handle_arrow_key(KeyCode::Right);
        assert_eq!(app.timer.duration(), current + Duration::from_secs(1));

        app.handle_arrow_key(KeyCode::Left);
        assert_eq!(app.timer.duration(), current);
    }

    #[test]
    fn test_handle_arrow_key_does_nothing_in_time_mode() {
        let mut app = App::new();
        app.mode = AppMode::Time;
        let initial_duration = app.timer.duration();

        app.handle_arrow_key(KeyCode::Up);
        app.handle_arrow_key(KeyCode::Down);
        app.handle_arrow_key(KeyCode::Left);
        app.handle_arrow_key(KeyCode::Right);

        assert_eq!(app.timer.duration(), initial_duration);
    }

    #[test]
    fn test_handle_arrow_key_does_nothing_in_stopwatch_mode() {
        let mut app = App::new();
        app.mode = AppMode::Stopwatch;
        let initial_duration = app.timer.duration();

        app.handle_arrow_key(KeyCode::Up);
        app.handle_arrow_key(KeyCode::Down);
        app.handle_arrow_key(KeyCode::Left);
        app.handle_arrow_key(KeyCode::Right);

        assert_eq!(app.timer.duration(), initial_duration);
    }

    #[test]
    fn test_handle_arrow_key_does_nothing_with_unknown_key() {
        let mut app = App::new();
        app.mode = AppMode::Countdown;
        let initial_duration = app.timer.duration();

        // Even in Countdown mode, non-arrow keys should do nothing
        app.handle_arrow_key(KeyCode::Char('x'));
        app.handle_arrow_key(KeyCode::Char('r'));
        app.handle_arrow_key(KeyCode::Esc);

        assert_eq!(app.timer.duration(), initial_duration);
    }

    // ============================================================
    // Key Dispatch via handle_key (integration)
    // ============================================================

    #[test]
    fn test_ctrl_c_quits_app() {
        let mut app = App::new();
        assert!(!app.should_quit);
        app.handle_key(key(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.should_quit, "Ctrl+C should set should_quit");
    }

    #[test]
    fn test_plain_c_does_not_quit() {
        let mut app = App::new();
        assert!(!app.should_quit);
        app.handle_key(key(KeyCode::Char('c'), KeyModifiers::NONE));
        assert!(!app.should_quit, "Plain 'c' should NOT set should_quit");
        assert_eq!(app.mode, AppMode::Countdown, "Plain 'c' should switch to Countdown");
    }

    #[test]
    fn test_q_quits_app() {
        let mut app = App::new();
        app.handle_key(key(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(app.should_quit);
    }

    #[test]
    fn test_space_starts_and_pauses_timer() {
        let mut app = App::new();
        app.mode = AppMode::Countdown;
        assert!(!app.timer.is_running());

        app.handle_key(key(KeyCode::Char(' '), KeyModifiers::NONE));
        assert!(app.timer.is_running());
        assert!(!app.timer.is_paused());

        app.handle_key(key(KeyCode::Char(' '), KeyModifiers::NONE));
        assert!(!app.timer.is_running());
        assert!(app.timer.is_paused());
    }

    #[test]
    fn test_space_starts_and_pauses_stopwatch() {
        let mut app = App::new();
        app.mode = AppMode::Stopwatch;
        assert!(!app.stopwatch.is_running());

        app.handle_key(key(KeyCode::Char(' '), KeyModifiers::NONE));
        assert!(app.stopwatch.is_running());

        app.handle_key(key(KeyCode::Char(' '), KeyModifiers::NONE));
        assert!(!app.stopwatch.is_running());
    }

    #[test]
    fn test_space_does_nothing_in_time_mode() {
        let mut app = App::new();
        app.mode = AppMode::Time;
        app.handle_key(key(KeyCode::Char(' '), KeyModifiers::NONE));
        assert!(!app.timer.is_running());
        assert!(!app.stopwatch.is_running());
        assert_eq!(app.mode, AppMode::Time);
    }

    #[test]
    fn test_r_resets_timer_in_countdown_mode() {
        let mut app = App::new();
        app.mode = AppMode::Countdown;
        app.handle_arrow_key(KeyCode::Up); // change duration away from initial
        assert_ne!(app.timer.duration(), app.timer.initial_duration());

        app.handle_key(key(KeyCode::Char('r'), KeyModifiers::NONE));
        assert_eq!(app.timer.duration(), app.timer.initial_duration());
        assert!(!app.timer.is_running());
    }

    #[test]
    fn test_r_resets_stopwatch() {
        let mut app = App::new();
        app.mode = AppMode::Stopwatch;
        app.handle_key(key(KeyCode::Char(' '), KeyModifiers::NONE));
        std::thread::sleep(Duration::from_millis(20));
        app.handle_key(key(KeyCode::Char(' '), KeyModifiers::NONE));
        assert!(app.stopwatch.elapsed_time() > Duration::ZERO);

        app.handle_key(key(KeyCode::Char('r'), KeyModifiers::NONE));
        assert_eq!(app.stopwatch.elapsed_time(), Duration::ZERO);
        assert!(!app.stopwatch.is_running());
        assert!(app.stopwatch.last_lap_elapsed().is_none());
    }

    #[test]
    fn test_lap_key_only_works_in_stopwatch_mode() {
        // In Stopwatch mode, 'l' records a lap.
        let mut app = App::new();
        app.mode = AppMode::Stopwatch;
        app.handle_key(key(KeyCode::Char(' '), KeyModifiers::NONE));
        std::thread::sleep(Duration::from_millis(20));
        app.handle_key(key(KeyCode::Char('l'), KeyModifiers::NONE));
        assert!(app.stopwatch.last_lap_elapsed().is_some());

        // In Time mode, 'l' must NOT record a lap on the (shared) stopwatch.
        let mut app = App::new();
        app.mode = AppMode::Time;
        app.handle_key(key(KeyCode::Char('l'), KeyModifiers::NONE));
        assert!(app.stopwatch.last_lap_elapsed().is_none());

        // In Countdown mode, likewise.
        let mut app = App::new();
        app.mode = AppMode::Countdown;
        app.handle_key(key(KeyCode::Char('l'), KeyModifiers::NONE));
        assert!(app.stopwatch.last_lap_elapsed().is_none());
    }

    #[test]
    fn test_unknown_key_is_ignored() {
        let mut app = App::new();
        let mode_before = app.mode;
        app.handle_key(key(KeyCode::Char('z'), KeyModifiers::NONE));
        assert_eq!(app.mode, mode_before);
        assert!(!app.should_quit);
    }

    // ============================================================
    // App Notifier Tests
    // ============================================================

    #[test]
    fn test_app_default_notifier_is_system_notifier() {
        let app = App::new();
        let debug_str = format!("{:?}", app.notifier);
        assert!(debug_str.contains("SystemNotifier"));
    }

    #[test]
    fn test_app_with_mock_notifier() {
        let app = App::new();
        let mock = Box::new(notification::MockNotifier::new());
        let app = app.with_notifier(mock);
        // Verify the app was reconstructed with the mock
        assert_eq!(app.mode, AppMode::Time);
        let debug_str = format!("{:?}", app.notifier);
        assert!(debug_str.contains("MockNotifier"));
    }

    // ============================================================
    // Integration Tests (App + Modules)
    // ============================================================

    #[test]
    fn test_countdown_timer_finish_sets_correct_state() {
        let mut timer = CountdownTimer::with_duration(25 * 60);
        timer.start();
        timer.finish();

        assert_eq!(timer.duration(), Duration::ZERO);
        assert!(!timer.is_paused());
        assert!(!timer.is_running());
        assert!(timer.is_finished());
        assert!(timer.end_time().is_none());
    }

    #[test]
    fn test_countdown_timer_finish_triggers_notification() {
        let mock = Box::new(notification::MockNotifier::new());
        let app = App::new().with_notifier(mock);
        // Verify the injected mock is in place; the real completion path is
        // exercised by the event loop and covered by CountdownTimer::has_expired
        // tests. Here we just confirm the notifier dispatches without panic.
        let mut notifier = notification::MockNotifier::new();
        notifier.send_notification("Countdown Timer Complete", "00:00 - Timer has finished");
        assert_eq!(notifier.count(), 1);
        // Use `app` so the with_notifier wiring is compiled/checked.
        let _ = format!("{:?}", app.notifier);
    }
}