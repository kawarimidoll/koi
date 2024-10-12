// 日本語🇯🇵の表示テスト
use std::io::Error;
use terminal::{CursorStyle, Event, KeyCode, KeyEvent, KeyModifiers, Terminal};
mod terminal;
use buffer::Buffer;
mod buffer;
mod position;
mod size;
use size::Size;
use view::View;
mod cursor;
mod line;
mod text_fragment;
mod view;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// TODO tabが含まれる場合の画面端の処理

// 将来的にはEditorは複数のViewとBufferを持つ
// それぞれのViewはBufferを参照する
// Editorは現在どのViewにフォーカスしているかの情報を持つ
pub struct Editor {
    should_quit: bool,
    // buffer: Buffer,
    views: Vec<View>,
    current_view_idx: usize,
    size: Size,
    message: Option<String>,
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
            let message = Some(format!("blank file"));
            (buffer, message)
        };

        let size = Terminal::size().unwrap_or_default();
        let view_size = Size {
            width: size.width,
            height: size.height.saturating_sub(1),
        };
        let view = View::new(buffer, view_size);
        Ok(Self {
            should_quit: false,
            views: vec![view],
            current_view_idx: 0,
            size,
            message,
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
        self.move_caret();

        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            match Terminal::read_event() {
                Ok(Event::Key(KeyEvent {
                    code, modifiers, ..
                })) => {
                    self.set_message(&format!(
                        "cursor: {}, screen: {}, off: {}, [{}], key: {}",
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
                        code,
                    ));
                    self.handle_key_event(code, modifiers);
                }
                Ok(Event::Resize(width16, height16)) => {
                    self.handle_resize_event(width16, height16);
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

    fn handle_key_event(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match (code, modifiers) {
            (KeyCode::Char('q'), KeyModifiers::NONE) => self.should_quit = true,
            (KeyCode::Char('i'), KeyModifiers::NONE) => self.insert_loop(),
            (KeyCode::Char('a'), KeyModifiers::NONE) => {
                self.current_view_mut().move_position(KeyCode::Right);
                self.insert_loop();
            }
            (KeyCode::Char('x'), KeyModifiers::NONE) => self.current_view_mut().remove_char(),
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => self.save(),

            (KeyCode::Left | KeyCode::Down | KeyCode::Right | KeyCode::Up, KeyModifiers::SHIFT)
            | (KeyCode::PageDown | KeyCode::PageUp, KeyModifiers::NONE) => {
                self.current_view_mut().scroll_screen(code);
            }
            (
                KeyCode::Left
                | KeyCode::Down
                | KeyCode::Right
                | KeyCode::Up
                | KeyCode::Home
                | KeyCode::End,
                KeyModifiers::NONE,
            ) => {
                self.current_view_mut().move_position(code);
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                self.current_view_mut().move_position(KeyCode::Left);
            }
            (KeyCode::Char('H'), KeyModifiers::SHIFT) => {
                self.current_view_mut().move_position(KeyCode::Home);
            }
            (KeyCode::Char('j'), KeyModifiers::NONE) => {
                self.current_view_mut().move_position(KeyCode::Down);
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) => {
                self.current_view_mut().move_position(KeyCode::Up);
            }
            (KeyCode::Char('l'), KeyModifiers::NONE) => {
                self.current_view_mut().move_position(KeyCode::Right);
            }
            (KeyCode::Char('L'), KeyModifiers::SHIFT) => {
                self.current_view_mut().move_position(KeyCode::End);
            }
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                self.current_view_mut().scroll_screen(KeyCode::PageDown);
            }
            (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                self.current_view_mut().scroll_screen(KeyCode::PageUp);
            }
            _ => (),
        }
    }

    fn save(&mut self) {
        if self.current_view().has_filename() {
            // TODO: handle error
            let save_result = self.current_view_mut().save();
            let message = if save_result.is_ok() {
                "File saved successfully"
            } else {
                "Error saving file"
            };
            self.set_message(message);
        } else {
            // TODO: input filename
        }
    }

    #[allow(clippy::as_conversions)]
    fn handle_resize_event(&mut self, width16: u16, height16: u16) {
        let width = width16 as usize;
        let height = height16 as usize;
        // let _ = Terminal::print_row(height - 1, &format!("Resize to: {width:?}, {height:?}"));
        self.size = Size { width, height };
        let view_size = Size {
            width,
            height: height.saturating_sub(1),
        };
        self.current_view_mut().set_size(view_size);
    }
    fn refresh_screen(&mut self) {
        if self.size.width == 0 || self.size.height == 0 {
            return;
        }
        let _ = Terminal::hide_caret();
        if let Some(line_text) = &self.message {
            let bottom_line = self.size.height.saturating_sub(1);
            Terminal::print_row(bottom_line, line_text).unwrap();
        }
        let _ = self.current_view_mut().render();
        self.move_caret();
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }
    fn move_caret(&self) {
        Terminal::move_caret_to(self.current_view().caret_screen_position()).unwrap();
    }

    // NOTE: easy version
    fn insert_loop(&mut self) {
        Terminal::set_cursor_style(CursorStyle::SteadyBar).unwrap();
        self.set_message("[ insert ]");
        loop {
            self.refresh_screen();
            if let Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) = Terminal::read_event()
            {
                match (code, modifiers) {
                    (KeyCode::Esc, _) => break,
                    (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                        self.current_view_mut().insert_char(c);
                        self.insert_message(&c.to_string());
                    }
                    (KeyCode::Tab, KeyModifiers::NONE) => {
                        self.current_view_mut().insert_char('\t');
                        self.insert_message("Tab");
                    }
                    (KeyCode::Enter, KeyModifiers::NONE) => {
                        self.current_view_mut().insert_char('\n');
                        self.insert_message("Enter");
                    }
                    (KeyCode::Delete, KeyModifiers::NONE) => {
                        self.current_view_mut().remove_char();
                        self.insert_message("Delete");
                    }
                    (KeyCode::Backspace, KeyModifiers::NONE) => {
                        // just detect if the caret is at the beginning of the buffer
                        // so we don't need to use caret_screen_position() here
                        if self.current_view().cursor.col_idx() > 0
                            || self.current_view().cursor.line_idx() > 0
                        {
                            self.current_view_mut().move_position(KeyCode::Left);
                            self.current_view_mut().remove_char();
                            self.insert_message("Backspace");
                        }
                    }
                    _ => (),
                }
            }
        }
        Terminal::set_cursor_style(CursorStyle::DefaultUserShape).unwrap();
    }
    fn insert_message(&mut self, input: &str) {
        let line = self
            .current_view()
            .get_line(self.current_view().cursor.line_idx())
            .map_or("no line", line::Line::content);

        self.set_message(&format!("[ insert ] input: {input}, content: {line}"));
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
            size: Size::default(),
        };
        assert_eq!(editor.size, Size::default());
        editor.handle_resize_event(10, 10);
        assert_eq!(editor.size, Size::new(10, 10));
    }
}
