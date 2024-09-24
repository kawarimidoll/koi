use crossterm::event::{
    read,
    Event::{self, Key},
    KeyCode::{self, Char, Down, End, Home, Left, PageDown, PageUp, Right, Up},
    KeyEvent,
};
use std::io::Error;
use terminal::{Position, Size};

use terminal::Terminal;
mod terminal;

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    cursor_position: Position,
}

impl Editor {
    pub fn run(&mut self) -> Result<(), Error> {
        Terminal::initialize()?;
        Terminal::print_row(0, "Hello, koi!")?;
        Terminal::print_row(1, "Type something. Press 'q' to quit.")?;
        Terminal::move_caret_to(self.cursor_position)?;

        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            match read() {
                Ok(event) => {
                    self.handle_event(&event)?;
                }
                Err(err) => {
                    let height = Terminal::size()?.height;
                    Terminal::print_row(height - 1, &format!("{err}"))?;
                }
            }
        }

        Terminal::terminate()?;
        Terminal::print_row(0, "Goodbye, koi!\r\n")?;
        Ok(())
    }
    fn handle_event(&mut self, event: &Event) -> Result<(), Error> {
        let height = Terminal::size()?.height;
        Terminal::print_row(height - 1, &format!("{event:?}"))?;
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
        {
            match code {
                Char('q') if *modifiers == crossterm::event::KeyModifiers::NONE => {
                    self.should_quit = true
                }

                Left | Down | Right | Up | Home | End | PageDown | PageUp => {
                    self.move_position(*code)?;
                }
                _ => (),
            }
        }
        Ok(())
    }
    fn refresh_screen(&mut self) -> Result<(), Error> {
        if self.should_quit {
            Terminal::clear_screen()?;
            Terminal::move_caret_to(Position::default())?;
        } else {
            Terminal::move_caret_to(self.cursor_position)?;
        }
        Terminal::execute()?;
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
