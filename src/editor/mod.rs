use crossterm::event::{
    read,
    Event::{self, Key},
    KeyCode::Char,
};
use std::io::Error;

use terminal::Terminal;
mod terminal;

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
}

impl Editor {
    pub fn run(&mut self) -> Result<(), Error> {
        Terminal::initialize()?;
        Terminal::print_row(0, "Hello, koi!")?;
        Terminal::print_row(1, "Type something. Press 'q' to quit.")?;

        loop {
            Terminal::execute()?;
            match read() {
                Ok(event) => {
                    self.handle_event(event)?;
                }
                Err(err) => {
                    let height = Terminal::size()?.height;
                    Terminal::print_row(height - 1, &format!("{err}"))?;
                }
            }
            if self.should_quit {
                break;
            }
        }

        Terminal::terminate()?;
        Terminal::print_row(0, "Goodbye, koi!")?;
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
}
