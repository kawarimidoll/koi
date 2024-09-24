use crossterm::event::{read, Event::Key, KeyCode::Char};

use terminal::Terminal;
mod terminal;

pub struct Editor {}

impl Editor {
    pub fn default() -> Self {
        Editor {}
    }

    pub fn run(&self) -> Result<(), std::io::Error> {
        Terminal::initialize()?;
        Terminal::print_row(0, "Hello, koi!")?;
        Terminal::print_row(1, "Type something. Press 'q' to quit.")?;

        let height = Terminal::size()?.height;

        loop {
            Terminal::execute()?;
            match read() {
                Ok(Key(event)) => {
                    Terminal::print_row(height - 1, &format!("{event:?}"))?;
                    if let Char(c) = event.code {
                        if c == 'q' {
                            break;
                        }
                    }
                }
                Err(err) => {
                    Terminal::print_row(height - 1, &format!("{err}"))?;
                }
                _ => (),
            }
        }

        Terminal::terminate()?;
        Terminal::print_row(0, "Goodbye, koi!")?;
        Ok(())
    }
}
