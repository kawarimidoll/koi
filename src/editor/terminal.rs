use crossterm::cursor::MoveTo;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType};
use crossterm::{queue, Command};
use std::io::{stdout, Error};

#[derive(Default)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

pub struct Terminal {}

impl Terminal {
    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_caret_to(Position::default())?;
        Ok(())
    }
    pub fn terminate() -> Result<(), Error> {
        disable_raw_mode()?;
        Ok(())
    }
    pub fn clear_screen() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::All))
    }
    pub fn move_caret_to(position: Position) -> Result<(), Error> {
        Self::queue_command(MoveTo(position.col as u16, position.row as u16))
    }
    fn queue_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)
    }
}
