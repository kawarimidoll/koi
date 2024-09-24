use crossterm::event::{
    read,
    Event::{self, Key},
    KeyCode::Char,
};
use std::io::Error;
use terminal::Position;

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
                    self.handle_event(event)?;
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
    fn handle_event(&mut self, event: Event) -> Result<(), Error> {
        let height = Terminal::size()?.height;
        Terminal::print_row(height - 1, &format!("{event:?}"))?;
        match event {
            Key(event) => match event.code {
                Char('q') => self.should_quit = true,
                _ => (),
            },
            _ => (),
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
}
