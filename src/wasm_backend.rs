use ratatui::{
    backend::Backend,
    buffer::Cell,
    layout::{Rect, Size},
    style::{Color, Modifier},
};
use std::io;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn xterm_write(data: &str);
}

pub struct WasmBackend {
    width: u16,
    height: u16,
    buf: String,
}

impl WasmBackend {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height, buf: String::with_capacity(4096) }
    }

    pub fn set_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    fn ansi_color(c: Color, fg: bool) -> String {
        let (fg_prefix, bg_prefix) = if fg { ("38", "39") } else { ("48", "49") };
        match c {
            Color::Reset       => format!("\x1b[{}m", bg_prefix),
            Color::Black       => format!("\x1b[{}m", if fg { "30" } else { "40" }),
            Color::Red         => format!("\x1b[{}m", if fg { "31" } else { "41" }),
            Color::Green       => format!("\x1b[{}m", if fg { "32" } else { "42" }),
            Color::Yellow      => format!("\x1b[{}m", if fg { "33" } else { "43" }),
            Color::Blue        => format!("\x1b[{}m", if fg { "34" } else { "44" }),
            Color::Magenta     => format!("\x1b[{}m", if fg { "35" } else { "45" }),
            Color::Cyan        => format!("\x1b[{}m", if fg { "36" } else { "46" }),
            Color::Gray        => format!("\x1b[{}m", if fg { "37" } else { "47" }),
            Color::DarkGray    => format!("\x1b[{}m", if fg { "90" } else { "100" }),
            Color::LightRed    => format!("\x1b[{}m", if fg { "91" } else { "101" }),
            Color::LightGreen  => format!("\x1b[{}m", if fg { "92" } else { "102" }),
            Color::LightYellow => format!("\x1b[{}m", if fg { "93" } else { "103" }),
            Color::LightBlue   => format!("\x1b[{}m", if fg { "94" } else { "104" }),
            Color::LightMagenta=> format!("\x1b[{}m", if fg { "95" } else { "105" }),
            Color::LightCyan   => format!("\x1b[{}m", if fg { "96" } else { "106" }),
            Color::White       => format!("\x1b[{}m", if fg { "97" } else { "107" }),
            Color::Rgb(r, g, b)=> format!("\x1b[{};2;{};{};{}m", fg_prefix, r, g, b),
            Color::Indexed(i)  => format!("\x1b[{};5;{}m", fg_prefix, i),
        }
    }

    fn cell_to_ansi(cell: &Cell) -> String {
        let mut s = String::new();
        s.push_str("\x1b[0m");
        let m = cell.modifier;
        if m.contains(Modifier::BOLD)       { s.push_str("\x1b[1m"); }
        if m.contains(Modifier::DIM)        { s.push_str("\x1b[2m"); }
        if m.contains(Modifier::ITALIC)     { s.push_str("\x1b[3m"); }
        if m.contains(Modifier::UNDERLINED) { s.push_str("\x1b[4m"); }
        if m.contains(Modifier::REVERSED)   { s.push_str("\x1b[7m"); }
        if m.contains(Modifier::HIDDEN)     { s.push_str("\x1b[8m"); }
        if m.contains(Modifier::CROSSED_OUT){ s.push_str("\x1b[9m"); }
        s.push_str(&Self::ansi_color(cell.fg, true));
        s.push_str(&Self::ansi_color(cell.bg, false));
        s.push_str(cell.symbol());
        s
    }
}

impl Backend for WasmBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        let mut last_row = u16::MAX;
        let mut last_col = u16::MAX;
        for (col, row, cell) in content {
            // Only move cursor if not on the next cell in sequence
            if row != last_row || col != last_col + 1 {
                self.buf.push_str(&format!("\x1b[{};{}H", row + 1, col + 1));
            }
            self.buf.push_str(&Self::cell_to_ansi(cell));
            last_row = row;
            last_col = col;
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.buf.push_str("\x1b[?25l");
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.buf.push_str("\x1b[?25h");
        Ok(())
    }

    fn get_cursor_position(&mut self) -> io::Result<ratatui::layout::Position> {
        Ok(ratatui::layout::Position { x: 0, y: 0 })
    }

    fn set_cursor_position<P: Into<ratatui::layout::Position>>(&mut self, position: P) -> io::Result<()> {
        let p = position.into();
        self.buf.push_str(&format!("\x1b[{};{}H", p.y + 1, p.x + 1));
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        self.buf.push_str("\x1b[2J\x1b[H");
        Ok(())
    }

    fn size(&self) -> io::Result<Size> {
        Ok(Size { width: self.width, height: self.height })
    }

    fn window_size(&mut self) -> io::Result<ratatui::backend::WindowSize> {
        Ok(ratatui::backend::WindowSize {
            columns_rows: Size { width: self.width, height: self.height },
            pixels: Size { width: 0, height: 0 },
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        if !self.buf.is_empty() {
            xterm_write(&self.buf);
            self.buf.clear();
        }
        Ok(())
    }
}
