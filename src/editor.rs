use crossterm::event::{read, Event::Key, KeyCode::Char};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

pub struct Editor {}

impl Editor {
    pub fn default() -> Self {
        Editor {}
    }

    pub fn run(&self) -> Result<(), std::io::Error> {
        enable_raw_mode()?;

        loop {
            match read() {
                Ok(Key(event)) => {
                    println!("{event:?}\r");
                    if let Char(c) = event.code {
                        if c == 'q' {
                            break;
                        }
                    }
                }
                Err(err) => println!("{err}"),
                _ => (),
            }
        }

        disable_raw_mode()?;
        Ok(())
    }
}
