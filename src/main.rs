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

struct App {
    should_quit: bool,
    font: LargeFont,
}

impl App {
    fn new() -> Self {
        Self {
            should_quit: false,
            font: LargeFont::new(),
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

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
                    .split(size);

                let now = Local::now();
                let time_str = now.format("%I:%M:%S %p").to_string();
                let date_str = now.format("%A, %B %d, %Y").to_string();

                self.render_large_text(f, chunks[0], &time_str, Color::White);

                // Render date using large text as well, but potentially smaller scale
                self.render_large_text(f, chunks[1], &date_str, Color::Yellow);
            })?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if key.code == KeyCode::Char('q') {
                        self.should_quit = true;
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
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
