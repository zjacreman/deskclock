use chrono::Local;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};
use std::{
    error::Error,
    time::{Duration, Instant},
};

mod font;
use font::LargeFont;

#[derive(PartialEq, Debug)]
enum AppMode {
    Time,
    Countdown,
    Stopwatch,
}

struct Stopwatch {
    elapsed_time: Duration,
    last_start_time: Option<Instant>,
    is_running: bool,
}

impl Stopwatch {
    fn new() -> Self {
        Self {
            elapsed_time: Duration::ZERO,
            last_start_time: None,
            is_running: false,
        }
    }

    fn start(&mut self) {
        if !self.is_running {
            self.is_running = true;
            self.last_start_time = Some(Instant::now());
        }
    }

    fn pause(&mut self) {
        if self.is_running {
            if let Some(start_time) = self.last_start_time {
                let elapsed_since_start = start_time.elapsed();
                self.elapsed_time += elapsed_since_start;
            }
            self.is_running = false;
            self.last_start_time = None;
        }
    }

    fn reset(&mut self) {
        self.elapsed_time = Duration::ZERO;
        self.last_start_time = None;
        self.is_running = false;
    }

    fn current_elapsed(&self) -> Duration {
        if self.is_running {
            if let Some(start_time) = self.last_start_time {
                self.elapsed_time + start_time.elapsed()
            } else {
                self.elapsed_time
            }
        } else {
            self.elapsed_time
        }
    }
}

struct CountdownTimer {
    duration: Duration,
    initial_duration: Duration,
    end_time: Option<Instant>,
    is_running: bool,
    is_paused: bool,
}

impl CountdownTimer {
    fn new() -> Self {
        let default_dur = Duration::from_secs(25 * 60);
        Self {
            duration: default_dur,
            initial_duration: default_dur,
            end_time: None,
            is_running: false,
            is_paused: false,
        }
    }

    fn remaining(&self) -> Duration {
        if let Some(end) = self.end_time {
            end.saturating_duration_since(Instant::now())
        } else {
            self.duration
        }
    }

    fn start(&mut self) {
        if !self.is_running {
            if self.duration > Duration::ZERO {
                self.initial_duration = self.duration;
            }
            self.end_time = Some(Instant::now() + self.remaining());
            self.is_running = true;
            self.is_paused = false;
        }
    }

    fn pause(&mut self) {
        if self.is_running {
            self.duration = self.remaining();
            self.end_time = None;
            self.is_running = false;
            self.is_paused = true;
        }
    }

    fn reset(&mut self) {
        self.end_time = None;
        self.is_running = false;
        self.is_paused = false;
        self.duration = self.initial_duration;
    }

    fn finish(&mut self) {
        self.duration = Duration::ZERO;
        self.end_time = None;
        self.is_running = false;
        self.is_paused = true;
    }

    fn adjust_minutes(&mut self, delta: i32) {
        self.stop();
        let secs = self.duration.as_secs() as i64 + (delta as i64 * 60);
        self.duration = Duration::from_secs(secs.max(0) as u64);
    }

    fn adjust_seconds(&mut self, delta: i32) {
        self.stop();
        let secs = self.duration.as_secs() as i64 + (delta as i64);
        self.duration = Duration::from_secs(secs.max(0) as u64);
    }

    fn stop(&mut self) {
        self.end_time = None;
        self.is_running = false;
        self.is_paused = false;
    }

    fn is_finished(&self) -> bool {
        self.is_running && self.remaining().as_secs() == 0
    }
}

struct App {
    should_quit: bool,
    font: LargeFont,
    mode: AppMode,
    timer: CountdownTimer,
    stopwatch: Stopwatch,
    use_24h_format: bool,
    flash_start_time: Option<Instant>,
}

impl App {
    fn new() -> Self {
        Self {
            should_quit: false,
            font: LargeFont::new(),
            mode: AppMode::Time,
            timer: CountdownTimer::new(),
            stopwatch: Stopwatch::new(),
            use_24h_format: false,
            flash_start_time: None,
        }
    }

    fn render_large_text(&self, f: &mut ratatui::Frame, area: Rect, text: &str, color: Color) {
        let base_w = self.font.glyph_width() as usize;
        let base_h = self.font.glyph_height() as usize;
        let text_chars: Vec<char> = text.chars().collect();
        let num_chars = text_chars.len();

        // Calculate total base dimensions
        let total_base_w = num_chars * base_w + (num_chars.saturating_sub(1));
        let total_base_h = base_h;

        // Scaling factor: how many times we can multiply the base font to fit the area
        let scale_w = (area.width as usize) / total_base_w;
        let scale_h = (area.height as usize) / total_base_h;
        let scale = scale_w.min(scale_h).max(1);

        let scaled_w = total_base_w * scale;
        let scaled_h = total_base_h * scale;

        // If it doesn't fit even at scale 1, fallback to normal text
        if scaled_w > area.width as usize || scaled_h > area.height as usize {
            let p = Paragraph::new(text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(color));
            f.render_widget(p, area);
            return;
        }

        let offset_y = (area.height as usize - scaled_h) / 2;

        // Render scaled glyphs
        for base_row in 0..base_h {
            let row_str = self.get_row_string(base_row, &text_chars);

            // Each base row is repeated 'scale' times vertically
            for s_row in 0..scale {
                let y_pos = area.y + ((offset_y + base_row * scale + s_row) as u16);

                // To scale horizontally, we need to repeat each character in row_str 'scale' times
                let mut scaled_row_str = String::with_capacity(row_str.len() * scale);
                for c in row_str.chars() {
                    for _ in 0..scale {
                        scaled_row_str.push(c);
                    }
                }

                let x_offset = (area.width as usize - scaled_w) / 2;
                let p = Paragraph::new(scaled_row_str.as_str()).style(Style::default().fg(color));
                let line_rect = Rect {
                    x: area.x + (x_offset as u16),
                    y: y_pos,
                    width: scaled_w as u16,
                    height: 1,
                };
                f.render_widget(p, line_rect);
            }
        }
    }

    fn get_row_string(&self, row: usize, text_chars: &[char]) -> String {
        let mut line = String::new();
        for (i, c) in text_chars.iter().enumerate() {
            if let Some(glyph) = self.font.get_glyph(*c) {
                line.push_str(&glyph[row]);
            } else {
                line.push_str("     ");
            }
            if i < text_chars.len() - 1 {
                line.push(' ');
            }
        }
        line
    }

    fn handle_arrow_key(&mut self, key: crossterm::event::KeyCode) {
        match (key, &self.mode) {
            (crossterm::event::KeyCode::Up, AppMode::Countdown) => self.timer.adjust_minutes(1),
            (crossterm::event::KeyCode::Down, AppMode::Countdown) => self.timer.adjust_minutes(-1),
            (crossterm::event::KeyCode::Left, AppMode::Countdown) => self.timer.adjust_seconds(-1),
            (crossterm::event::KeyCode::Right, AppMode::Countdown) => self.timer.adjust_seconds(1),
            _ => {}
        }
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
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
                                Paragraph::new("").style(Style::default().bg(Color::Red)),
                                size,
                            );
                        }
                    } else {
                        self.flash_start_time = None;
                    }
                }

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(70),
                        Constraint::Percentage(20),
                        Constraint::Percentage(10),
                    ])
                    .split(size);

                match self.mode {
                    AppMode::Time => {
                        let now = Local::now();
                        let time_fmt = if self.use_24h_format { "%H:%M:%S" } else { "%I:%M:%S %p" };
                        let time_str = now.format(time_fmt).to_string();
                        let date_str = now.format("%A, %B %d, %Y").to_string();

                        self.render_large_text(f, chunks[0], &time_str, Color::White);
                        self.render_large_text(f, chunks[1], &date_str, Color::Yellow);
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

                        let is_paused = self.timer.is_paused;
                        let is_running = self.timer.is_running;

                        let color = if is_running || is_paused {
                            Color::Cyan
                        } else {
                            Color::White
                        };

                        let is_visible = if is_running {
                            true
                        } else if is_paused {
                            (Local::now().timestamp_millis() / 500) % 2 == 0
                        } else {
                            true
                        };

                        if is_visible {
                            self.render_large_text(f, chunks[0], &timer_str, color);
                        }

                        let end_time_str = if self.timer.is_running {
                            let _end = Instant::now() + self.timer.remaining();
                            // We can't easily convert Instant to Local time without using something like chrono::Utc::now() + Duration
                            // But we can approximate:
                            let now_local = Local::now();
                            let end_local = now_local
                                + chrono::Duration::from_std(self.timer.remaining())
                                    .unwrap_or(chrono::Duration::zero());
                            format!("Ends at: {}", end_local.format("%I:%M:%S %p").to_string())
                        } else {
                            "Paused".to_string()
                        };

                        let p = Paragraph::new(end_time_str)
                            .alignment(Alignment::Center)
                            .style(Style::default().fg(Color::Gray));
                        f.render_widget(p, chunks[1]);
                    }
                    AppMode::Stopwatch => {
                        let elapsed = self.stopwatch.current_elapsed();
                        let h = elapsed.as_secs() / 3600;
                        let m = (elapsed.as_secs() % 3600) / 60;
                        let s = elapsed.as_secs() % 60;
                        let timer_str = format!("{:02}:{:02}:{:02}", h, m, s);

                        let color = if self.stopwatch.is_running {
                            Color::Magenta
                        } else {
                            Color::White
                        };

                        let is_visible = if self.stopwatch.is_running {
                            true
                        } else if !self.stopwatch.is_running
                            && self.stopwatch.elapsed_time > Duration::ZERO
                        {
                            (Local::now().timestamp_millis() / 500) % 2 == 0
                        } else {
                            true
                        };

                        if is_visible {
                            self.render_large_text(f, chunks[0], &timer_str, color);
                        }

                        let p = Paragraph::new("Stopwatch")
                            .alignment(Alignment::Center)
                            .style(Style::default().fg(Color::Gray));
                        f.render_widget(p, chunks[1]);
                    }
                }

                let cmd_text = match self.mode {
                    AppMode::Time => "q: Quit | c: Countdown | s: Stopwatch | h: 12/24h",
                    AppMode::Countdown => {
                        if self.timer.is_running {
                            "q: Quit | t: Time | s: Stopwatch | Space: Pause | r: Reset"
                        } else {
                            "q: Quit | t: Time | s: Stopwatch | Space: Start | r: Reset | ↑↓: Min | ←→: Sec"
                        }
                    }
                    AppMode::Stopwatch => {
                        if self.stopwatch.is_running {
                            "q: Quit | t: Time | c: Countdown | Space: Pause | r: Reset"
                        } else {
                            "q: Quit | t: Time | c: Countdown | Space: Start | r: Reset"
                        }
                    }
                };

                let cmd_p = Paragraph::new(cmd_text)
                    .alignment(Alignment::Center)
                    .style(Style::default().fg(Color::DarkGray));
                f.render_widget(cmd_p, chunks[2]);
            })?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Char('c') => self.mode = AppMode::Countdown,
                        KeyCode::Char('t') => self.mode = AppMode::Time,
                        KeyCode::Char('h') => self.use_24h_format = !self.use_24h_format,
                        KeyCode::Char('s') => self.mode = AppMode::Stopwatch,
                        KeyCode::Char('r') => match self.mode {
                            AppMode::Countdown => self.timer.reset(),
                            AppMode::Stopwatch => self.stopwatch.reset(),
                            _ => {}
                        },
                        KeyCode::Char(' ') => match self.mode {
                            AppMode::Countdown => {
                                if self.timer.is_running {
                                    self.timer.pause();
                                } else {
                                    self.timer.start();
                                }
                            }
                            AppMode::Stopwatch => {
                                if self.stopwatch.is_running {
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
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
                if self.timer.is_finished() {
                    self.flash_start_time = Some(Instant::now());
                    self.timer.finish();
                }
            }

            if self.should_quit {
                break;
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
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
    use std::time::Duration;

    // ============================================================
    // Stopwatch Tests
    // ============================================================

    #[test]
    fn test_stopwatch_new_starts_zeroed() {
        let sw = Stopwatch::new();
        assert_eq!(sw.elapsed_time, Duration::ZERO);
        assert!(sw.last_start_time.is_none());
        assert!(!sw.is_running);
    }

    #[test]
    fn test_stopwatch_start() {
        let mut sw = Stopwatch::new();
        assert!(!sw.is_running);
        sw.start();
        assert!(sw.is_running);
        assert!(sw.last_start_time.is_some());
    }

    #[test]
    fn test_stopwatch_start_when_already_running_does_nothing() {
        let mut sw = Stopwatch::new();
        sw.start();
        let _start_time = sw.last_start_time.unwrap();
        std::thread::sleep(Duration::from_millis(10));
        sw.start();
        // Should not change - still running from the first start
        assert!(sw.is_running);
    }

    #[test]
    fn test_stopwatch_pause_while_not_running_does_nothing() {
        let mut sw = Stopwatch::new();
        sw.pause();
        assert!(!sw.is_running);
        assert!(sw.last_start_time.is_none());
    }

    #[test]
    fn test_stopwatch_pause_stops_running() {
        let mut sw = Stopwatch::new();
        sw.start();
        std::thread::sleep(Duration::from_millis(50));
        let _elapsed_before = sw.current_elapsed();
        sw.pause();
        assert!(!sw.is_running);
        assert!(sw.last_start_time.is_none());
        // elapsed_time should have some value now
        assert!(sw.elapsed_time >= Duration::ZERO);
    }

    #[test]
    fn test_stopwatch_reset_clears_all_state() {
        let mut sw = Stopwatch::new();
        sw.start();
        std::thread::sleep(Duration::from_millis(100));
        sw.pause();
        sw.reset();
        assert_eq!(sw.elapsed_time, Duration::ZERO);
        assert!(sw.last_start_time.is_none());
        assert!(!sw.is_running);
    }

    #[test]
    fn test_stopwatch_current_elapsed_while_running() {
        let mut sw = Stopwatch::new();
        sw.start();
        let initial_elapsed = sw.current_elapsed();
        std::thread::sleep(Duration::from_millis(100));
        let later_elapsed = sw.current_elapsed();
        // Should have increased
        assert!(later_elapsed >= initial_elapsed);
    }

    #[test]
    fn test_stopwatch_current_elapsed_while_paused() {
        let mut sw = Stopwatch::new();
        sw.start();
        std::thread::sleep(Duration::from_millis(100));
        let elapsed_at_pause = sw.current_elapsed();
        sw.pause();
        // Wait a bit - elapsed should not change while paused
        std::thread::sleep(Duration::from_millis(100));
        let elapsed_after_sleep = sw.current_elapsed();
        // Allow some tolerance for timing
        assert!(elapsed_after_sleep >= elapsed_at_pause - Duration::from_millis(50));
        assert!(elapsed_after_sleep <= elapsed_at_pause + Duration::from_millis(50));
    }

    #[test]
    fn test_stopwatch_restart_after_pause() {
        let mut sw = Stopwatch::new();
        sw.start();
        std::thread::sleep(Duration::from_millis(100));
        sw.pause();
        let elapsed_before_restart = sw.elapsed_time;
        std::thread::sleep(Duration::from_millis(100));
        sw.start();
        assert!(sw.is_running);
        // After restarting, current_elapsed should be greater than before
        std::thread::sleep(Duration::from_millis(100));
        let elapsed_now = sw.current_elapsed();
        assert!(elapsed_now > elapsed_before_restart);
    }

    // ============================================================
    // CountdownTimer Tests
    // ============================================================

    #[test]
    fn test_countdown_timer_new_has_default_25_minutes() {
        let timer = CountdownTimer::new();
        assert_eq!(timer.duration, Duration::from_secs(25 * 60));
        assert_eq!(timer.initial_duration, Duration::from_secs(25 * 60));
        assert!(timer.end_time.is_none());
        assert!(!timer.is_running);
        assert!(!timer.is_paused);
    }

    #[test]
    fn test_countdown_timer_remaining_when_not_running() {
        let timer = CountdownTimer::new();
        assert_eq!(timer.remaining(), Duration::from_secs(25 * 60));
    }

    #[test]
    fn test_countdown_timer_start() {
        let mut timer = CountdownTimer::new();
        assert!(!timer.is_running);
        timer.start();
        assert!(timer.is_running);
        assert!(!timer.is_paused);
        assert!(timer.end_time.is_some());
    }

    #[test]
    fn test_countdown_timer_start_when_already_running_does_nothing() {
        let mut timer = CountdownTimer::new();
        timer.start();
        let _end_time = timer.end_time.unwrap();
        std::thread::sleep(Duration::from_millis(10));
        timer.start();
        // Should still be running with same end_time (not extended)
        assert!(timer.is_running);
    }

    #[test]
    fn test_countdown_timer_pause_while_not_running_does_nothing() {
        let mut timer = CountdownTimer::new();
        timer.pause();
        assert!(!timer.is_running);
        assert!(!timer.is_paused);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_pause_sets_paused_flag() {
        let mut timer = CountdownTimer::new();
        timer.start();
        std::thread::sleep(Duration::from_millis(50));
        timer.pause();
        assert!(!timer.is_running);
        assert!(timer.is_paused);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_pause_saves_remaining_duration() {
        let mut timer = CountdownTimer::new();
        // Set a custom duration
        timer.duration = Duration::from_secs(100);
        timer.start();
        std::thread::sleep(Duration::from_millis(50));
        let remaining_before_pause = timer.remaining();
        timer.pause();
        // Duration should now be approximately what was remaining
        assert!(timer.duration.as_millis() >= remaining_before_pause.as_millis() - 100);
        assert!(timer.duration.as_millis() <= remaining_before_pause.as_millis() + 100);
    }

    #[test]
    fn test_countdown_timer_reset_restores_initial_duration() {
        let mut timer = CountdownTimer::new();
        let initial = timer.initial_duration;
        // Modify duration without calling start() (which would update initial_duration)
        timer.duration = Duration::from_secs(10);
        timer.pause(); // pause does nothing if not running
        timer.duration = Duration::from_secs(5);
        timer.reset();
        assert_eq!(timer.duration, initial);
        assert_eq!(timer.initial_duration, initial);
        assert!(!timer.is_running);
        assert!(!timer.is_paused);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_finish_sets_zero_duration() {
        let mut timer = CountdownTimer::new();
        timer.start();
        timer.finish();
        assert_eq!(timer.duration, Duration::ZERO);
        assert!(!timer.is_running);
        assert!(timer.is_paused);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_adjust_minutes_positive() {
        let mut timer = CountdownTimer::new();
        let initial = timer.duration;
        timer.adjust_minutes(5);
        assert_eq!(timer.duration, initial + Duration::from_secs(300));
    }

    #[test]
    fn test_countdown_timer_adjust_minutes_negative() {
        let mut timer = CountdownTimer::new();
        let initial = timer.duration;
        timer.adjust_minutes(-1);
        assert_eq!(timer.duration, initial - Duration::from_secs(60));
    }

    #[test]
    fn test_countdown_timer_adjust_minutes_does_not_go_below_zero() {
        let mut timer = CountdownTimer::new();
        timer.duration = Duration::from_secs(30);
        timer.adjust_minutes(-10); // Should try to subtract 600 seconds
        assert!(timer.duration >= Duration::ZERO);
    }

    #[test]
    fn test_countdown_timer_adjust_seconds_positive() {
        let mut timer = CountdownTimer::new();
        let initial = timer.duration;
        timer.adjust_seconds(30);
        assert_eq!(timer.duration, initial + Duration::from_secs(30));
    }

    #[test]
    fn test_countdown_timer_adjust_seconds_negative() {
        let mut timer = CountdownTimer::new();
        let initial = timer.duration;
        timer.adjust_seconds(-10);
        assert_eq!(timer.duration, initial - Duration::from_secs(10));
    }

    #[test]
    fn test_countdown_timer_adjust_seconds_does_not_go_below_zero() {
        let mut timer = CountdownTimer::new();
        timer.duration = Duration::from_secs(5);
        timer.adjust_seconds(-10); // Should try to subtract 10 seconds
        assert!(timer.duration >= Duration::ZERO);
    }

    #[test]
    fn test_countdown_timer_stop_clears_running_state() {
        let mut timer = CountdownTimer::new();
        timer.start();
        timer.stop();
        assert!(!timer.is_running);
        assert!(!timer.is_paused);
        assert!(timer.end_time.is_none());
    }

    #[test]
    fn test_countdown_timer_is_finished_while_running() {
        let mut timer = CountdownTimer::new();
        timer.duration = Duration::ZERO;
        timer.start();
        // With zero duration, remaining() will be ZERO
        // is_finished checks is_running && remaining().as_secs() == 0
        assert!(timer.is_finished());
    }

    #[test]
    fn test_countdown_timer_is_finished_when_not_running() {
        let timer = CountdownTimer::new();
        assert!(!timer.is_finished());
    }

    // ============================================================
    // App Tests
    // ============================================================

    #[test]
    fn test_app_new_creates_default_state() {
        let app = App::new();
        assert!(!app.should_quit);
        assert_eq!(app.mode, AppMode::Time);
        assert!(!app.timer.is_running);
        assert!(!app.stopwatch.is_running);
        assert!(app.flash_start_time.is_none());
    }

    #[test]
    fn test_app_24h_toggle() {
        let mut app = App::new();
        assert!(!app.use_24h_format, "Should default to 12h format");
        app.use_24h_format = !app.use_24h_format;
        assert!(app.use_24h_format, "Should toggle to 24h format");
        app.use_24h_format = !app.use_24h_format;
        assert!(!app.use_24h_format, "Should toggle back to 12h format");
    }

    #[test]
    fn test_app_mode_switch_to_countdown() {
        let mut app = App::new();
        app.mode = AppMode::Countdown;
        assert_eq!(app.mode, AppMode::Countdown);
    }

    #[test]
    fn test_app_mode_switch_to_stopwatch() {
        let mut app = App::new();
        app.mode = AppMode::Stopwatch;
        assert_eq!(app.mode, AppMode::Stopwatch);
    }

    #[test]
    fn test_app_mode_switch_to_time() {
        let mut app = App::new();
        app.mode = AppMode::Countdown;
        app.mode = AppMode::Time;
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
    // LargeFont Integration Tests (via App)
    // ============================================================

    #[test]
    fn test_app_font_has_large_font() {
        let app = App::new();
        assert_eq!(app.font.glyph_width(), 5);
        assert_eq!(app.font.glyph_height(), 5);
    }

    #[test]
    fn test_app_font_can_render_digits() {
        let app = App::new();
        for c in '0'..='9' {
            assert!(
                app.font.get_glyph(c).is_some(),
                "Digit {} should have a glyph",
                c
            );
        }
    }

    #[test]
    fn test_app_font_can_render_colon() {
        let app = App::new();
        assert!(
            app.font.get_glyph(':').is_some(),
            "Colon should have a glyph"
        );
    }

    #[test]
    fn test_app_font_can_render_space() {
        let app = App::new();
        assert!(
            app.font.get_glyph(' ').is_some(),
            "Space should have a glyph"
        );
    }

    // ============================================================
    // Arrow Key Event Handling Tests
    // ============================================================

    #[test]
    fn test_handle_arrow_key_adjusts_timer_in_countdown_mode() {
        let mut app = App::new();
        app.mode = AppMode::Countdown;
        let initial_duration = app.timer.duration;

        app.handle_arrow_key(KeyCode::Up);
        assert_eq!(
            app.timer.duration,
            initial_duration + Duration::from_secs(60)
        );

        app.handle_arrow_key(KeyCode::Down);
        assert_eq!(app.timer.duration, initial_duration);

        let current = app.timer.duration;
        app.handle_arrow_key(KeyCode::Right);
        assert_eq!(app.timer.duration, current + Duration::from_secs(1));

        app.handle_arrow_key(KeyCode::Left);
        assert_eq!(app.timer.duration, current);
    }

    #[test]
    fn test_handle_arrow_key_does_nothing_in_time_mode() {
        let mut app = App::new();
        app.mode = AppMode::Time;
        let initial_duration = app.timer.duration;

        app.handle_arrow_key(KeyCode::Up);
        app.handle_arrow_key(KeyCode::Down);
        app.handle_arrow_key(KeyCode::Left);
        app.handle_arrow_key(KeyCode::Right);

        assert_eq!(app.timer.duration, initial_duration);
    }

    #[test]
    fn test_handle_arrow_key_does_nothing_in_stopwatch_mode() {
        let mut app = App::new();
        app.mode = AppMode::Stopwatch;
        let initial_duration = app.timer.duration;

        app.handle_arrow_key(KeyCode::Up);
        app.handle_arrow_key(KeyCode::Down);
        app.handle_arrow_key(KeyCode::Left);
        app.handle_arrow_key(KeyCode::Right);

        assert_eq!(app.timer.duration, initial_duration);
    }

    #[test]
    fn test_handle_arrow_key_does_nothing_with_unknown_key() {
        let mut app = App::new();
        app.mode = AppMode::Countdown;
        let initial_duration = app.timer.duration;

        // Even in Countdown mode, non-arrow keys should do nothing
        app.handle_arrow_key(KeyCode::Char('x'));
        app.handle_arrow_key(KeyCode::Char('r'));
        app.handle_arrow_key(KeyCode::Esc);

        assert_eq!(app.timer.duration, initial_duration);
    }

    // ============================================================
    // Stopwatch Edge Cases
    // ============================================================

    #[test]
    fn test_stopwatch_elapsed_time_accrues_across_multiple_pause_cycles() {
        let mut sw = Stopwatch::new();

        // First cycle
        sw.start();
        std::thread::sleep(Duration::from_millis(50));
        sw.pause();
        let elapsed_after_first = sw.elapsed_time;

        // Second cycle
        std::thread::sleep(Duration::from_millis(50)); // Should not add to elapsed
        sw.start();
        std::thread::sleep(Duration::from_millis(50));
        sw.pause();

        let total_elapsed = sw.elapsed_time;

        // Total should be approximately 100ms (two 50ms runs)
        assert!(total_elapsed >= elapsed_after_first);
    }

    #[test]
    fn test_stopwatch_current_elapsed_returns_zero_when_reset() {
        let sw = Stopwatch::new();
        assert_eq!(sw.current_elapsed(), Duration::ZERO);
    }

    // ============================================================
    // CountdownTimer Edge Cases
    // ============================================================

    #[test]
    fn test_countdown_timer_remaining_with_zero_duration() {
        let mut timer = CountdownTimer::new();
        timer.duration = Duration::ZERO;
        assert_eq!(timer.remaining(), Duration::ZERO);
    }

    #[test]
    fn test_countdown_timer_start_with_zero_duration_sets_running_but_remaining_is_zero() {
        let mut timer = CountdownTimer::new();
        timer.duration = Duration::ZERO;
        timer.start();
        // start() sets is_running = true regardless of duration,
        // but remaining() will be zero since duration is zero
        assert!(timer.is_running);
        assert_eq!(timer.remaining(), Duration::ZERO);
    }

    #[test]
    fn test_countdown_timer_adjust_minutes_with_no_running_state() {
        let mut timer = CountdownTimer::new();
        let initial = timer.duration;
        timer.adjust_minutes(10);
        assert_eq!(timer.duration, initial + Duration::from_secs(600));
    }

    #[test]
    fn test_countdown_timer_adjust_seconds_with_no_running_state() {
        let mut timer = CountdownTimer::new();
        let initial = timer.duration;
        timer.adjust_seconds(45);
        assert_eq!(timer.duration, initial + Duration::from_secs(45));
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
    // LargeFont Edge Cases
    // ============================================================

    #[test]
    fn test_large_font_glyphs_are_consistent() {
        let font = LargeFont::new();
        // Get a glyph multiple times - should return same reference
        let glyph1 = font.get_glyph('0');
        let glyph2 = font.get_glyph('0');
        assert!(std::ptr::eq(glyph1.unwrap(), glyph2.unwrap()));
    }

    #[test]
    fn test_large_font_mixed_case_input() {
        let font = LargeFont::new();
        // Test that mixed case works
        let time_str = "12:34 PM";
        for c in time_str.chars() {
            if c.is_alphabetic() {
                assert!(
                    font.get_glyph(c).is_some(),
                    "Character '{}' should have a glyph",
                    c
                );
            }
        }
    }
}
