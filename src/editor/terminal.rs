use super::position::Position;
use super::size::Size;
pub use crossterm::cursor::SetCursorStyle as CursorStyle;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::read;
pub use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::style::{
    Attribute::{Reset, Reverse},
    Print,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, DisableLineWrap, EnableLineWrap,
    EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
};
use crossterm::{queue, Command};
use std::io::{stdout, Error, Write};

pub struct Terminal {}

impl Terminal {
    pub fn initialize() -> Result<(), Error> {
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        Self::disable_line_wrap()?;
        Self::clear_screen()?;
        Self::execute()
    }
    pub fn terminate() -> Result<(), Error> {
        Self::leave_alternate_screen()?;
        Self::enable_line_wrap()?;
        Self::show_caret()?;
        Self::set_cursor_style(CursorStyle::DefaultUserShape)?;
        Self::execute()?;
        disable_raw_mode()
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
    pub fn set_title(title: &str) -> Result<(), Error> {
        Self::queue_command(SetTitle(title))
    }
    pub fn read_event() -> Result<Event, Error> {
        let event = read();
        match event {
            Ok(Event::Key(key_event)) => {
                // necessary for windows
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    event
                } else {
                    Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Ignore not-press event",
                    ))
                }
            }
            _ => event,
        }
    }
    pub fn set_cursor_style(style: CursorStyle) -> Result<(), Error> {
        Self::queue_command(style)
    }
    pub fn clear_screen() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::All))
    }
    fn clear_line() -> Result<(), Error> {
        Self::queue_command(Clear(ClearType::UntilNewLine))
    }
    pub fn move_caret_to(position: Position) -> Result<(), Error> {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        Self::queue_command(MoveTo(position.col_idx as u16, position.line_idx as u16))
    }
    pub fn hide_caret() -> Result<(), Error> {
        Self::queue_command(Hide)
    }
    pub fn show_caret() -> Result<(), Error> {
        Self::queue_command(Show)
    }
    pub fn print(string: &str) -> Result<(), Error> {
        Self::queue_command(Print(string))
    }
    pub fn print_row(line_idx: usize, line_text: &str) -> Result<(), Error> {
        let col_idx = 0;
        Self::move_caret_to(Position { line_idx, col_idx })?;
        Self::clear_line()?;
        Self::print(line_text)
    }
    pub fn print_invert_row(line_idx: usize, line_text: &str) -> Result<(), Error> {
        let width = Self::size()?.width;
        // :width$ -> pad to width
        // .width$ -> truncate to width
        Self::print_row(
            line_idx,
            &format!("{Reverse}{line_text:width$.width$}{Reset}"),
        )
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
