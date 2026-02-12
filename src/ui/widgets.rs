use macroquad::prelude::{BLACK, WHITE, draw_rectangle, draw_text, screen_height, screen_width};

pub fn draw_menu_box(lines: &[String]) {
    let width = screen_width() * 0.6;
    let height = 24.0 * (lines.len() as f32) + 32.0;
    let x = (screen_width() - width) * 0.5;
    let y = screen_height() / 2.0 - height / 2.0;
    draw_rectangle(x, y, width, height, BLACK);
    let mut offset = y + 30.0;
    for line in lines {
        draw_text(line, x + 12.0, offset, 24.0, WHITE);
        offset += 26.0;
    }
}

pub fn format_name_with_cursor(name: &str, cursor_pos: usize) -> String {
    let chars: Vec<char> = name.chars().collect();
    let cursor = cursor_pos.min(chars.len());
    let mut result = String::with_capacity(chars.len() + 1);
    for (i, &ch) in chars.iter().enumerate() {
        if i == cursor {
            result.push('_');
        }
        result.push(ch);
    }
    if cursor == chars.len() {
        result.push('_');
    }
    result
}

