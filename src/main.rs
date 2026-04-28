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

#[derive(PartialEq)]
enum AppMode {
    Time,
    Countdown,
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
    flash_start_time: Option<Instant>,
}

impl App {
    fn new() -> Self {
        Self {
            should_quit: false,
            font: LargeFont::new(),
            mode: AppMode::Time,
            timer: CountdownTimer::new(),
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
                        let time_str = now.format("%I:%M:%S %p").to_string();
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
                }

                let cmd_text = match self.mode {
                    AppMode::Time => "q: Quit | c: Countdown",
                    AppMode::Countdown => {
                        if self.timer.is_running {
                            "q: Quit | t: Time | Space: Pause | r: Reset"
                        } else {
                            "q: Quit | t: Time | Space: Start | r: Reset | ↑↓: Min | ←→: Sec"
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
                        KeyCode::Char('r') => self.timer.reset(),
                        KeyCode::Char(' ') => {
                            if self.timer.is_running {
                                self.timer.pause();
                            } else {
                                self.timer.start();
                            }
                        }
                        KeyCode::Up => self.timer.adjust_minutes(1),
                        KeyCode::Down => self.timer.adjust_minutes(-1),
                        KeyCode::Left => self.timer.adjust_seconds(-1),
                        KeyCode::Right => self.timer.adjust_seconds(1),
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
