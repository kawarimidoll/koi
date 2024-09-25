use crossterm::event::{
    read,
    Event::{Key, Resize},
    KeyCode::{self, Char, Down, End, Home, Left, PageDown, PageUp, Right, Up},
    KeyEvent, KeyModifiers,
};
use std::{cmp::min, fs::read_to_string, io::Error};
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
    location: Location,
    scroll_offset: Position,
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
        self.move_caret();

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

        match code {
            Char('q') if modifiers == KeyModifiers::NONE => self.should_quit = true,

            Left | Down | Right | Up | Home | End | PageDown | PageUp => {
                self.move_position(code);
            }
            _ => (),
        }
        let Location { x, y } = self.location;
        let Position { col, row } = self.text_location_to_position();
        let Position {
            col: off_c,
            row: off_r,
        } = self.scroll_offset;
        let _ = Terminal::print_row(
            height - 1,
            &format!("loc: {x},{y}, pos: {col},{row}, off: {off_c},{off_r}"),
        );
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
        self.move_caret();
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }
    pub fn render(&mut self, size: Size) -> Result<(), Error> {
        // render function
        if !self.needs_redraw {
            return Ok(());
        }
        let Size { width, height } = size;
        let top = self.scroll_offset.row;
        let left = self.scroll_offset.col;
        let right = left.saturating_add(width);
        for current_row in 0..height.saturating_sub(1) {
            let current_line = top.saturating_add(current_row);
            if let Some(line) = self.lines.get(current_line) {
                let end = min(right, line.len());
                Terminal::print_row(current_row, line.get(left..end).unwrap_or_default())?;
                continue;
            }
            Terminal::print_row(current_row, "~")?;
        }
        // the bottom line is reserved for messages
        self.needs_redraw = false;
        Ok(())
    }
    fn move_caret(&self) {
        let cursor_position = self
            .text_location_to_position()
            .saturating_sub(&self.scroll_offset);
        Terminal::move_caret_to(cursor_position).unwrap();
    }
    fn text_location_to_position(&self) -> Position {
        Position {
            col: min(self.location.x, self.get_current_line_len()),
            row: min(self.location.y, self.get_lines_count()),
        }
    }
    fn move_position(&mut self, code: KeyCode) {
        match code {
            Left => self.move_left(),
            Right => self.move_right(),
            Up => self.move_up(),
            Down => self.move_down(),
            Home => self.location.x = 0,
            End => self.location.x = usize::MAX,
            PageUp => self.location.y = 0,
            PageDown => self.location.y = self.size.height,
            _ => (),
        };
        self.scroll_into_view();
    }
    fn get_current_line_len(&self) -> usize {
        self.lines.get(self.location.y).map_or(0, String::len)
    }
    fn get_lines_count(&self) -> usize {
        self.lines.len()
    }
    #[allow(clippy::arithmetic_side_effects)]
    // allow this because check boundary condition by myself
    fn move_left(&mut self) {
        self.location.x = self.text_location_to_position().col;
        if self.location.x > 0 {
            self.location.x -= 1;
        } else if self.location.y > 0 {
            self.move_up();
            self.location.x = self.get_current_line_len();
        }
    }
    fn move_right(&mut self) {
        self.location.x = self.text_location_to_position().col;
        if self.location.x < self.get_current_line_len() {
            self.location.x = self.location.x.saturating_add(1);
        } else if self.location.y < self.get_lines_count() {
            self.move_down();
            self.location.x = 0;
        }
    }
    #[allow(clippy::arithmetic_side_effects)]
    // allow this because check boundary condition by myself
    fn move_up(&mut self) {
        if self.location.y > 0 {
            self.location.y -= 1;
        }
    }
    fn move_down(&mut self) {
        if self.location.y < self.get_lines_count() {
            self.location.y = self.location.y.saturating_add(1);
        }
    }
    fn scroll_into_view(&mut self) {
        let Position { col, row } = self.text_location_to_position();
        let Size { width, height } = self.size;
        // horizontal
        if col < self.scroll_offset.col {
            self.scroll_offset.col = col;
            self.needs_redraw = true;
        } else if col >= self.scroll_offset.col.saturating_add(width) {
            self.scroll_offset.col = col.saturating_add(1).saturating_sub(width);
            self.needs_redraw = true;
        }
        // vertical
        if row < self.scroll_offset.row {
            self.scroll_offset.row = row;
            self.needs_redraw = true;
        } else if row >= self.scroll_offset.row.saturating_add(height) {
            self.scroll_offset.row = row.saturating_add(1).saturating_sub(height);
            self.needs_redraw = true;
        }
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
