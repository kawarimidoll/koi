// 日本語🇯🇵の表示テスト
use std::io::Error;
use terminal::{CursorStyle, Event, KeyCode, KeyEvent, KeyModifiers, Terminal};
mod terminal;
use buffer::Buffer;
mod buffer;
use position::Position;
mod position;
use command_bar::CommandBar;
mod command_bar;
mod size;
use size::Size;
use view::{MoveCode, ScrollCode, View};
mod cursor;
use status_bar::{DocumentStatus, StatusBar};
mod file_info;
mod line;
mod status_bar;
mod text_fragment;
mod view;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO tabが含まれる場合の画面端の処理

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
    Command,
    Search,
}

// 将来的にはEditorは複数のViewとBufferを持つ
// それぞれのViewはBufferを参照する
// Editorは現在どのViewにフォーカスしているかの情報を持つ
pub struct Editor {
    should_quit: bool,
    // buffer: Buffer,
    views: Vec<View>,
    current_view_idx: usize,
    mode: Mode,
    size: Size,
    message: Option<String>,
    command_bar: Option<CommandBar>,
    status_bar: StatusBar,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?;
        Terminal::set_title(&format!("{NAME} - {VERSION}"))?;

        let args: Vec<String> = std::env::args().collect();
        // only load the first file for now
        let (buffer, message) = if let Some(first) = args.get(1) {
            let buffer = Buffer::from_file(first);
            let message = Some(format!("Load file: {first}"));
            (buffer, message)
        } else {
            let buffer = Buffer::default();
            let message = Some("blank file".to_string());
            (buffer, message)
        };

        let size = Terminal::size().unwrap_or_default();
        let view_size = Size {
            width: size.width,
            // -1 for status bar
            // -1 for command / message bar
            height: size.height.saturating_sub(2),
        };
        let view = View::new(buffer, view_size);
        Ok(Self {
            should_quit: false,
            views: vec![view],
            current_view_idx: 0,
            mode: Mode::Normal,
            size,
            message,
            command_bar: None,
            status_bar: StatusBar::new(size.width),
        })
    }

    fn current_view(&self) -> &View {
        self.views.get(self.current_view_idx).unwrap()
    }
    fn current_view_mut(&mut self) -> &mut View {
        self.views.get_mut(self.current_view_idx).unwrap()
    }

    fn set_message(&mut self, message: &str) {
        self.message = Some(message.to_string());
    }
    pub fn run(&mut self) {
        // let mut last_cursor = Cursor::default();
        loop {
            if self.should_quit {
                break;
            }
            let status = DocumentStatus::from(self);
            self.status_bar.update_status(status);
            self.refresh_screen();
            match Terminal::read_event() {
                Ok(Event::Key(KeyEvent {
                    code, modifiers, ..
                })) => {
                    match self.mode {
                        Mode::Normal => self.handle_key_event_nomal(code, modifiers),
                        Mode::Insert => self.handle_key_event_insert(code, modifiers),
                        Mode::Command | Mode::Search => {
                            self.handle_key_event_command(code, modifiers)
                        }
                    }
                    // if last_cursor != self.current_view().cursor {
                    //     cursor moved
                    // }
                }
                Ok(Event::Resize(width16, height16)) => {
                    self.handle_resize_event(width16, height16);
                    self.set_message(&format!("Resize to: {}", self.size));
                }
                Err(err) => {
                    self.set_message(&format!("{err}"));
                }
                _ => {
                    self.set_message("Unsupported event!");
                }
            }
        }
    }

    fn key_to_string(code: KeyCode, modifiers: KeyModifiers) -> String {
        let mut result = match code {
            KeyCode::Char(' ') => "Space".to_string(),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Enter => "CR".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::Backspace => "BS".to_string(),
            KeyCode::Delete => "Del".to_string(),
            _ => format!("{code:?}"),
        };

        if modifiers.contains(KeyModifiers::SHIFT) && result.len() > 1 {
            result.insert_str(0, "S-");
        }
        if modifiers.contains(KeyModifiers::ALT) {
            result.insert_str(0, "A-");
        }
        if modifiers.contains(KeyModifiers::CONTROL) {
            result.insert_str(0, "C-");
        }

        if result.len() > 1 {
            format!("<{}>", result.to_uppercase())
        } else {
            result
        }
    }

    fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;

        match self.mode {
            Mode::Normal => {
                Terminal::set_cursor_style(CursorStyle::DefaultUserShape).unwrap();
                self.command_bar = None;
            }
            Mode::Insert => {
                Terminal::set_cursor_style(CursorStyle::SteadyBar).unwrap();
            }
            Mode::Command | Mode::Search => {
                Terminal::set_cursor_style(CursorStyle::SteadyBar).unwrap();
                self.command_bar = Some(CommandBar::new(self.mode));
            }
        }
    }

    fn show_cursor_info(&mut self) {
        self.set_message(&format!(
            "cursor: {}, screen: {}, off: {}, [{}]",
            self.current_view().cursor,
            self.current_view().caret_screen_position(),
            self.current_view().offset,
            self.current_view()
                .get_fragment_by_position(self.current_view().cursor.position())
                .map(|fragment| {
                    format!(
                        "{}, {}, {}",
                        fragment,
                        fragment.width(),
                        fragment.left_col_width()
                    )
                })
                .unwrap_or_default(),
        ));
    }

    fn handle_key_event_nomal(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let key_repr: &str = &Editor::key_to_string(code, modifiers);
        match key_repr {
            "q" => self.quit_with_confirm(),
            "Q" => self.should_quit = true,
            "i" => self.set_mode(Mode::Insert),
            "a" => {
                self.current_view_mut().move_position(MoveCode::Right);
                self.set_mode(Mode::Insert);
            }
            "I" => {
                self.current_view_mut().move_position(MoveCode::FirstChar);
                self.set_mode(Mode::Insert);
            }
            "A" => {
                self.current_view_mut().move_position(MoveCode::LastChar);
                self.set_mode(Mode::Insert);
            }
            "o" => {
                self.current_view_mut().move_position(MoveCode::LastChar);
                self.set_mode(Mode::Insert);
                self.current_view_mut().insert_char('\n');
            }
            "O" => {
                self.current_view_mut().move_position(MoveCode::FirstChar);
                self.set_mode(Mode::Insert);
                self.current_view_mut().insert_char_without_move('\n');
            }
            "x" => self.current_view_mut().remove_char(),
            "<C-G>" => self.show_cursor_info(),
            "<C-S>" => self.save(),
            ":" => self.set_mode(Mode::Command),
            "/" => self.set_mode(Mode::Search),
            "n" => {
                self.set_message("TODO: search next");
            }
            "N" => {
                self.set_message("TODO: search prev");
            }

            "<S-LEFT>" => {
                self.current_view_mut().scroll_screen(ScrollCode::Left(1));
            }
            "<S-RIGHT>" => {
                self.current_view_mut().scroll_screen(ScrollCode::Right(1));
            }
            "<S-DOWN>" => {
                self.current_view_mut().scroll_screen(ScrollCode::Down(1));
            }
            "<S-UP>" => {
                self.current_view_mut().scroll_screen(ScrollCode::Up(1));
            }
            "<PAGEDOWN>" | "<C-F>" => {
                let height = self.current_view().height();
                self.current_view_mut()
                    .scroll_screen(ScrollCode::Down(height));
            }
            "<PAGEUP>" | "<C-B>" => {
                let height = self.current_view().height();
                self.current_view_mut()
                    .scroll_screen(ScrollCode::Up(height));
            }
            "<LEFT>" | "h" => {
                self.current_view_mut().move_position(MoveCode::Left);
            }
            "<HOME>" | "0" => {
                self.current_view_mut().move_position(MoveCode::FirstChar);
            }
            "H" => {
                self.current_view_mut()
                    .move_position(MoveCode::FirstNonBlank);
            }
            "<DOWN>" | "j" => {
                self.current_view_mut().move_position(MoveCode::Down);
            }
            "<UP>" | "k" => {
                self.current_view_mut().move_position(MoveCode::Up);
            }
            "<RIGHT>" | "l" => {
                self.current_view_mut().move_position(MoveCode::Right);
            }
            "<END>" | "L" => {
                self.current_view_mut().move_position(MoveCode::LastChar);
            }
            "g" => {
                self.current_view_mut().move_position(MoveCode::FirstLine);
            }
            "G" => {
                self.current_view_mut().move_position(MoveCode::LastLine);
            }
            _ => (),
        }
    }

    fn save(&mut self) {
        if self.current_view().has_path() {
            let message = match self.current_view_mut().save() {
                Ok(()) => "File saved successfully",
                Err(_) => "Error: Saving file failed",
            };
            self.set_message(message);
        } else {
            self.set_message("Error: No file name");
        }
    }
    fn save_as(&mut self, filename: &str) {
        let message = match self.current_view_mut().save_as(filename) {
            Ok(()) => "File saved successfully",
            Err(_) => "Error: Saving file failed",
        };
        self.set_message(message);
    }
    fn quit_with_confirm(&mut self) {
        if self.current_view().buffer.modified_count == 0 {
            self.should_quit = true;
        } else {
            self.set_message("Unsaved changes.");
        }
    }

    #[allow(clippy::as_conversions)]
    fn handle_resize_event(&mut self, width16: u16, height16: u16) {
        let width = width16 as usize;
        let height = height16 as usize;
        self.size = Size { width, height };
        let view_size = Size {
            width,
            // -1 for status bar
            // -1 for command / message bar
            height: height.saturating_sub(2),
        };
        self.current_view_mut().set_size(view_size);
        self.status_bar.set_size(width);
    }
    fn refresh_screen(&mut self) {
        if self.size.width == 0 || self.size.height == 0 {
            return;
        }
        let _ = Terminal::hide_caret();
        let _ = self.current_view_mut().render();
        let _ = self.status_bar.render(self.size.height.saturating_sub(2));
        if self.command_bar.is_some() {
            let bottom_line = self.size.height.saturating_sub(1);
            let command_bar = self.command_bar.as_mut().unwrap();
            command_bar.render(bottom_line).unwrap();
            Terminal::move_caret_to(Position {
                col_idx: command_bar.caret_col,
                line_idx: bottom_line,
            })
            .unwrap();
        } else {
            if let Some(line_text) = &self.message {
                let bottom_line = self.size.height.saturating_sub(1);
                Terminal::print_row(bottom_line, line_text).unwrap();
            }
            self.move_caret();
        }
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }
    fn move_caret(&self) {
        Terminal::move_caret_to(self.current_view().caret_screen_position()).unwrap();
    }

    // NOTE: easy version
    fn handle_key_event_insert(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match (code, modifiers) {
            (KeyCode::Esc, _) => {
                self.set_mode(Mode::Normal);
            }
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.current_view_mut().insert_char(c);
            }
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.current_view_mut().insert_char('\t');
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.current_view_mut().insert_char('\n');
            }
            (KeyCode::Delete, KeyModifiers::NONE) => {
                self.current_view_mut().remove_char();
            }
            (KeyCode::Backspace, KeyModifiers::NONE)
            | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                // just detect if the caret is at the beginning of the buffer
                // so we don't need to use caret_screen_position() here
                if self.current_view().cursor.col_idx() > 0
                    || self.current_view().cursor.line_idx() > 0
                {
                    self.current_view_mut().move_position(MoveCode::Left);
                    self.current_view_mut().remove_char();
                }
            }
            _ => (),
        }
    }

    fn handle_key_event_command(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match (code, modifiers) {
            (KeyCode::Esc, _) => {
                self.set_mode(Mode::Normal);
            }
            (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                self.command_bar.as_mut().unwrap().insert(c);
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                let value = self.command_bar.as_ref().unwrap().value().to_string();
                if self.command_bar.as_ref().unwrap().mode == Mode::Command {
                    self.run_command(&value.trim());
                } else {
                    self.current_view().search(&value);
                    self.set_message(&format!("search: {value}"));
                }
                self.set_mode(Mode::Normal);
            }
            (KeyCode::Backspace, KeyModifiers::NONE)
            | (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                self.command_bar.as_mut().unwrap().delete_backward();
            }
            _ => (),
        }
    }

    fn run_command(&mut self, prompt: &str) {
        let (command, args) = prompt.split_once(' ').unwrap_or((prompt, ""));

        match command {
            "q" | "quit" => self.quit_with_confirm(),
            "q!" | "quit!" => self.should_quit = true,
            "w" | "write" => {
                if args.is_empty() {
                    self.save();
                } else {
                    self.save_as(args);
                }
            }
            "echo" => self.set_message(args),
            _ => self.set_message(&format!("Unknown command: {command}")),
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print("Goodbye, koi!\r\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize() {
        let buffer = Buffer::from_string("this\nis\ntest.\n");
        let size = Size::new(10, 10);
        let view = View::new(buffer, size);
        let mut editor = Editor {
            should_quit: false,
            views: vec![view],
            current_view_idx: 0,
            mode: Mode::Normal,
            size: Size::default(),
            message: None,
            command_bar: None,
            status_bar: StatusBar::default(),
        };
        assert_eq!(editor.size, Size::default());
        editor.handle_resize_event(10, 10);
        assert_eq!(editor.size, Size::new(10, 10));
    }

    #[test]
    fn test_key_to_string() {
        let str = Editor::key_to_string(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(str, "a");
        let str = Editor::key_to_string(KeyCode::Char('A'), KeyModifiers::SHIFT);
        assert_eq!(str, "A");
        let str = Editor::key_to_string(KeyCode::Char('a'), KeyModifiers::CONTROL);
        assert_eq!(str, "<C-A>");
        let str = Editor::key_to_string(KeyCode::Char('a'), KeyModifiers::ALT);
        assert_eq!(str, "<A-A>");
        let str = Editor::key_to_string(
            KeyCode::Char('a'),
            KeyModifiers::ALT | KeyModifiers::CONTROL,
        );
        assert_eq!(str, "<C-A-A>");
        let str = Editor::key_to_string(KeyCode::Char(' '), KeyModifiers::NONE);
        assert_eq!(str, "<SPACE>");
        let str = Editor::key_to_string(KeyCode::Left, KeyModifiers::NONE);
        assert_eq!(str, "<LEFT>");
        let str = Editor::key_to_string(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(str, "<CR>");
        let str = Editor::key_to_string(KeyCode::Enter, KeyModifiers::SHIFT);
        assert_eq!(str, "<S-CR>");
    }
}
