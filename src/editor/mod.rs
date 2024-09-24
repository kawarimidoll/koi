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
        println!("Hello, koi!\r");
        println!("Type something. Press 'q' to quit.\r");

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

        Terminal::terminate()?;
        println!("Goodbye, koi!");
        Ok(())
    }
}
