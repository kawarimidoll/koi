use crossterm::cursor::MoveTo;
use crossterm::event::{read, Event::Key, KeyCode::Char};
use crossterm::queue;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use std::io::stdout;

pub struct Editor {}

impl Editor {
    pub fn default() -> Self {
        Editor {}
    }

    pub fn run(&self) -> Result<(), std::io::Error> {
        Self::initialize()?;
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

        Self::terminate()?;
        println!("Goodbye, koi!");
        Ok(())
    }

    fn initialize() -> Result<(), std::io::Error> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor_to(0, 0)?;
        Ok(())
    }
    fn terminate() -> Result<(), std::io::Error> {
        disable_raw_mode()?;
        Ok(())
    }
    fn clear_screen() -> Result<(), std::io::Error> {
        queue!(stdout(), Clear(ClearType::All))?;
        Ok(())
    }
    fn move_cursor_to(x: u16, y: u16) -> Result<(), std::io::Error> {
        queue!(stdout(), MoveTo(x, y))?;
        Ok(())
    }
}
