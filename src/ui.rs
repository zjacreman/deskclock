use crate::font::LargeFont;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::Paragraph,
};

pub fn create_main_layout(area: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(70),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
        ])
        .split(area)
        .to_vec()
}

pub fn render_large_text(f: &mut Frame, area: Rect, text: &str, color: Color, font: &LargeFont) {
    let base_w = font.glyph_width() as usize;
    let base_h = font.glyph_height() as usize;
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
        let row_str = get_row_string(base_row, &text_chars, font);

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

fn get_row_string(row: usize, text_chars: &[char], font: &LargeFont) -> String {
    let mut line = String::new();
    for (i, c) in text_chars.iter().enumerate() {
        if let Some(glyph) = font.get_glyph(*c) {
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
