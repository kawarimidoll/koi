use crossterm::event::{
    read,
    Event::Key,
    KeyCode::{self, Char, Down, End, Home, Left, PageDown, PageUp, Right, Up},
    KeyEvent, KeyModifiers,
};
use std::{fs::read_to_string, io::Error};
use terminal::{Position, Size};

use terminal::Terminal;
mod terminal;

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    cursor_position: Position,
    lines: Vec<String>,
    needs_redraw: bool,
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

    pub fn run(&mut self) -> Result<(), Error> {
        Terminal::initialize()?;
        self.handle_args();
        self.needs_redraw = true;
        let bottom_line = Terminal::size()?.height.saturating_sub(1);
        Terminal::print_row(bottom_line, "Type something. Press 'q' to quit.")?;
        Terminal::move_caret_to(self.cursor_position)?;

        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            match read() {
                Ok(Key(key_event)) => {
                    // necessary for windows
                    if key_event.kind == crossterm::event::KeyEventKind::Press {
                        self.handle_key_event(&key_event)?;
                    }
                }
                Err(err) => {
                    Terminal::print_row(bottom_line, &format!("{err}"))?;
                }
                _ => {
                    Terminal::print_row(bottom_line, "Unsupported event!")?;
                }
            }
        }

        Terminal::terminate()?;
        Terminal::print_row(0, "Goodbye, koi!\r\n")?;
        Ok(())
    }
    fn handle_key_event(&mut self, event: &KeyEvent) -> Result<(), Error> {
        let height = Terminal::size()?.height;
        let KeyEvent {
            code, modifiers, ..
        } = event;
        Terminal::print_row(height - 1, &format!("code: {code:?}, mod: {modifiers:?}"))?;

        match code {
            Char('q') if *modifiers == KeyModifiers::NONE => self.should_quit = true,

            Left | Down | Right | Up | Home | End | PageDown | PageUp => {
                self.move_position(*code)?;
            }
            _ => (),
        }
        Ok(())
    }
    fn refresh_screen(&mut self) -> Result<(), Error> {
        Terminal::hide_caret()?;
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::move_caret_to(Position::default())?;
        } else {
            self.render()?;
            Terminal::move_caret_to(self.cursor_position)?;
        }
        Terminal::show_caret()?;
        Terminal::execute()?;
        Ok(())
    }
    pub fn render(&mut self) -> Result<(), Error> {
        // render function
        if !self.needs_redraw {
            return Ok(());
        }
        let Size { width, height } = Terminal::size()?;
        for current_row in 0..height.saturating_sub(1) {
            if let Some(line) = self.lines.get(current_row) {
                let mut l = String::from(line);
                l.truncate(width);
                Terminal::print_row(current_row, &l)?;
                continue;
            }
            Terminal::print_row(current_row, "~\r\n")?;
        }
        // the bottom line is reserved for messages
        self.needs_redraw = false;
        Ok(())
    }
    fn move_position(&mut self, code: KeyCode) -> Result<(), Error> {
        let Size { width, height } = Terminal::size()?;
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
        Ok(())
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
