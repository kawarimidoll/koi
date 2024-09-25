use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, DisableLineWrap, EnableLineWrap,
    EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{queue, Command};
use std::io::{stdout, Error, Write};

#[derive(Clone, Copy, Default)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}
#[derive(Clone, Copy, Default)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

pub struct Terminal {}

impl Terminal {
    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        Self::disable_line_wrap()?;
        Self::clear_screen()?;
        Self::execute()?;
        Ok(())
    }
    pub fn terminate() -> Result<(), Error> {
        Self::leave_alternate_screen()?;
        Self::enable_line_wrap()?;
        Self::show_caret()?;
        Self::execute()?;
        disable_raw_mode()?;
        Ok(())
    }
    fn enter_alternate_screen() -> Result<(), Error> {
        Self::queue_command(EnterAlternateScreen)
    }
    fn leave_alternate_screen() -> Result<(), Error> {
        Self::queue_command(LeaveAlternateScreen)
    }
    fn enable_line_wrap() -> Result<(), Error> {
        Self::queue_command(EnableLineWrap)
    }
    fn disable_line_wrap() -> Result<(), Error> {
        Self::queue_command(DisableLineWrap)
    }
    pub fn clear_screen() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::All))
    }
    fn clear_line() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::UntilNewLine))
    }
    pub fn move_caret_to(position: Position) -> Result<(), Error> {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        Self::queue_command(MoveTo(position.col as u16, position.row as u16))
    }
    pub fn hide_caret() -> Result<(), Error> {
        Self::queue_command(Hide)
    }
    pub fn show_caret() -> Result<(), Error> {
        Self::queue_command(Show)
    }
    fn print(string: &str) -> Result<(), Error> {
        Self::queue_command(Print(string))
    }
    pub fn print_row(row: usize, line_text: &str) -> Result<(), Error> {
        Self::move_caret_to(Position { col: 0, row })?;
        Self::clear_line()?;
        Self::print(line_text)
    }
    #[allow(clippy::as_conversions)]
    pub fn size() -> Result<Size, Error> {
        let (width16, height16) = size()?;
        let width = width16 as usize;
        let height = height16 as usize;
        Ok(Size { width, height })
    }
    pub fn execute() -> Result<(), Error> {
        stdout().flush()
    }
    fn queue_command<T: Command>(command: T) -> Result<(), Error> {
        queue!(stdout(), command)
    }
}
