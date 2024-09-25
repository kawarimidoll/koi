use crossterm::event::{
    read,
    Event::{Key, Resize},
    KeyCode::{self, Char, Down, End, Home, Left, PageDown, PageUp, Right, Up},
    KeyEvent, KeyModifiers,
};
use std::{cmp, fs::read_to_string, io::Error};
use terminal::{Position, Size};

use terminal::Terminal;
mod terminal;

#[derive(Copy, Clone, Default)]
pub struct Location {
    // the position of the document
    pub x: usize,
    pub y: usize,
}

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    cursor_position: Position,
    scroll_offset: Location,
    lines: Vec<String>,
    needs_redraw: bool,
    size: Size,
}

impl Editor {
    fn handle_args(&mut self) {
        let args: Vec<String> = std::env::args().collect();
        // only load the first file for now
        if let Some(first) = args.get(1) {
            if let Ok(lines) = Self::load(first) {
                self.lines = lines;
            }
        }
    }
    pub fn load(filename: &str) -> Result<Vec<String>, Error> {
        let contents = read_to_string(filename)?;
        let mut lines = Vec::new();
        for line in contents.lines() {
            lines.push(String::from(line));
        }
        Ok(lines)
    }

    pub fn run(&mut self) {
        Terminal::initialize().unwrap();
        self.handle_args();
        self.needs_redraw = true;
        self.size = Terminal::size().unwrap_or_default();
        let result = self.repl();
        Terminal::terminate().unwrap();
        result.unwrap();
    }

    pub fn repl(&mut self) -> Result<(), Error> {
        let bottom_line = self.size.height.saturating_sub(1);
        Terminal::print_row(bottom_line, "Type something. Press 'q' to quit.")?;
        Terminal::move_caret_to(self.cursor_position)?;

        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            match read() {
                Ok(Key(key_event)) => {
                    // necessary for windows
                    if key_event.kind == crossterm::event::KeyEventKind::Press {
                        self.handle_key_event(key_event);
                    }
                }
                Ok(Resize(width16, height16)) => {
                    self.handle_resize_event(width16, height16);
                }
                Err(err) => {
                    Terminal::print_row(bottom_line, &format!("{err}"))?;
                }
                _ => {
                    Terminal::print_row(bottom_line, "Unsupported event!")?;
                }
            }
        }
        Ok(())
    }
    fn handle_key_event(&mut self, event: KeyEvent) {
        let height = self.size.height;
        let KeyEvent {
            code, modifiers, ..
        } = event;
        let _ = Terminal::print_row(height - 1, &format!("code: {code:?}, mod: {modifiers:?}"));

        match code {
            Char('q') if modifiers == KeyModifiers::NONE => self.should_quit = true,

            Left | Down | Right | Up | Home | End | PageDown | PageUp => {
                self.move_position(code);
            }
            _ => (),
        }
    }
    #[allow(clippy::as_conversions)]
    fn handle_resize_event(&mut self, width16: u16, height16: u16) {
        let width = width16 as usize;
        let height = height16 as usize;
        let _ = Terminal::print_row(height - 1, &format!("Resize to: {width:?}, {height:?}"));
        self.size = Size { width, height };
        self.needs_redraw = true;
    }
    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();
        let _ = self.render(self.size);
        let _ = Terminal::move_caret_to(self.cursor_position);
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }
    pub fn render(&mut self, size: Size) -> Result<(), Error> {
        // render function
        if !self.needs_redraw {
            return Ok(());
        }
        let Size { width, height } = size;
        let top = self.scroll_offset.y;
        let left = self.scroll_offset.x;
        let right = left.saturating_add(width);
        for current_row in 0..height.saturating_sub(1) {
            let current_line = top.saturating_add(current_row);
            if let Some(line) = self.lines.get(current_line) {
                let end = cmp::min(right, line.len());
                Terminal::print_row(current_row, line.get(left..end).unwrap_or_default())?;
                continue;
            }
            Terminal::print_row(current_row, "~")?;
        }
        // the bottom line is reserved for messages
        self.needs_redraw = false;
        Ok(())
    }
    fn move_position(&mut self, code: KeyCode) {
        let Size { width, height } = self.size;
        match code {
            Left if self.cursor_position.col > 0 => self.cursor_position.col -= 1,
            Right if self.cursor_position.col < width => self.cursor_position.col += 1,
            Up if self.cursor_position.row > 0 => self.cursor_position.row -= 1,
            Down if self.cursor_position.row < height => self.cursor_position.row += 1,
            Home => self.cursor_position.col = 0,
            End => self.cursor_position.col = width,
            PageUp => self.cursor_position.row = 0,
            PageDown => self.cursor_position.row = height,
            _ => (),
        };
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print_row(0, "Goodbye, koi!\r\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let result = Editor::load("tests/fixtures/load.md");
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "# this is test file for load");
        assert_eq!(lines[1], "");
        assert_eq!(lines[2], "this is sample text");
    }
}
