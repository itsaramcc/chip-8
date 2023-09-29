#![allow(dead_code)]

use std::io::Write;

pub const SHOW_CURSOR: &str = "\x1B[?25h";
pub const HIDE_CURSOR: &str = "\x1B[?25l";
pub const CLEAR_TERMINAL: &str = "\x1B[2J";
pub const RESET_FORMATTING: &str = "\x1B[0m";
pub const CURSOR_TOP_LEFT: &str = "\x1B[;H";

pub fn show_cursor() {
    print!("{SHOW_CURSOR}");
}

pub fn hide_cursor() {
    print!("{HIDE_CURSOR}");
}

pub fn clear() {
    print!("{CLEAR_TERMINAL}");
}

#[derive(Clone, Copy)]
pub struct Color(u8, u8, u8);

impl Color {
    pub fn from_u32(val: u32) -> Color {
        Color(((val & 0xFF0000) >> 16) as u8, ((val & 0x00FF00) >> 8) as u8, (val & 0x0000FF) as u8)
    }
}

fn draw(foreground: Color, background: Color) -> String {
    let foreground_color = format!("\x1B[38;2;{};{};{}m", foreground.0, foreground.1, foreground.2);
    let background_color = format!("\x1B[48;2;{};{};{}m", background.0, background.1, background.2);

    format!("{}{}▀{}", foreground_color, background_color, RESET_FORMATTING)
}

pub fn initialize() {
	print!("{CLEAR_TERMINAL}");			// Clear Screen
}


pub fn render(canvas: Vec<Vec<Color>>) { 

    let mut buffer = "".to_string();

    for y in (0..canvas.len()).step_by(2) {
        for x in 0..canvas[y].len() {
            buffer += &draw(canvas[y][x], canvas[y+1][x]);
        }
        buffer += "\n";
    }

    print!("{CURSOR_TOP_LEFT}");
    std::io::stdout().flush().unwrap();
    print!("{buffer}");
     
    buffer.clear();
}