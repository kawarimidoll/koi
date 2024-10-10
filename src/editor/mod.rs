// 日本語🇯🇵の表示テスト
use std::io::Error;
use terminal::{CursorStyle, Event, KeyCode, KeyEvent, KeyModifiers, Terminal};
mod terminal;
// use buffer::Buffer;
mod buffer;
mod position;
mod size;
use size::Size;
use view::View;
mod cursor;
mod line;
mod text_fragment;
mod view;

// TODO tabが含まれる場合の画面端の処理

// 将来的にはEditorは複数のViewとBufferを持つ
// それぞれのViewはBufferを参照する
// Editorは現在どのViewにフォーカスしているかの情報を持つ
#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    // buffer: Buffer,
    view: View,
    size: Size,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?;
        let mut editor = Self::default();
        editor.view = View::new();
        editor.size = Terminal::size().unwrap_or_default();
        Ok(editor)
    }

    pub fn run(&mut self) {
        self.print_bottom("Type something. Press 'q' to quit.");
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
                    self.handle_key_event(code, modifiers);
                    self.print_bottom(&format!(
                        "pos: {}, cw: {}, off: {}, [{}], key: {}",
                        self.view.caret_screen_position(),
                        self.view.cursor.col_want(),
                        self.view.offset,
                        self.view
                            .get_fragment_by_position(self.view.cursor.position())
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
                }
                Ok(Event::Resize(width16, height16)) => {
                    self.handle_resize_event(width16, height16);
                }
                Err(err) => {
                    self.print_bottom(&format!("{err}"));
                }
                _ => {
                    self.print_bottom("Unsupported event!");
                }
            }
        }
    }

    // for debug
    fn print_bottom(&self, line_text: &str) {
        let bottom_line = self.size.height.saturating_sub(1);
        Terminal::print_row(bottom_line, line_text).unwrap();
    }

    fn handle_key_event(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        match (code, modifiers) {
            (KeyCode::Char('q'), KeyModifiers::NONE) => self.should_quit = true,
            (KeyCode::Char('i'), KeyModifiers::NONE) => self.insert_loop(),
            (KeyCode::Char('a'), KeyModifiers::NONE) => {
                self.view.move_position(self.size, KeyCode::Right);
                self.insert_loop();
            }
            (KeyCode::Char('x'), KeyModifiers::NONE) => self.view.remove_char(),

            (KeyCode::Left | KeyCode::Down | KeyCode::Right | KeyCode::Up, KeyModifiers::SHIFT)
            | (KeyCode::PageDown | KeyCode::PageUp, KeyModifiers::NONE) => {
                self.view.scroll_screen(self.size, code);
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
                self.view.move_position(self.size, code);
            }
            (KeyCode::Char('h'), KeyModifiers::NONE) => {
                self.view.move_position(self.size, KeyCode::Left);
            }
            (KeyCode::Char('H'), KeyModifiers::SHIFT) => {
                self.view.move_position(self.size, KeyCode::Home);
            }
            (KeyCode::Char('j'), KeyModifiers::NONE) => {
                self.view.move_position(self.size, KeyCode::Down);
            }
            (KeyCode::Char('k'), KeyModifiers::NONE) => {
                self.view.move_position(self.size, KeyCode::Up);
            }
            (KeyCode::Char('l'), KeyModifiers::NONE) => {
                self.view.move_position(self.size, KeyCode::Right);
            }
            (KeyCode::Char('L'), KeyModifiers::SHIFT) => {
                self.view.move_position(self.size, KeyCode::End);
            }
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                self.view.scroll_screen(self.size, KeyCode::PageDown);
            }
            (KeyCode::Char('b'), KeyModifiers::CONTROL) => {
                self.view.scroll_screen(self.size, KeyCode::PageUp);
            }
            _ => (),
        }
    }
    #[allow(clippy::as_conversions)]
    fn handle_resize_event(&mut self, width16: u16, height16: u16) {
        let width = width16 as usize;
        let height = height16 as usize;
        // let _ = Terminal::print_row(height - 1, &format!("Resize to: {width:?}, {height:?}"));
        self.size = Size { width, height };
        self.view.ensure_redraw();
    }
    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();
        let _ = self.view.render(self.size);
        self.move_caret();
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }
    fn move_caret(&self) {
        Terminal::move_caret_to(self.view.caret_screen_position()).unwrap();
    }

    // NOTE: easy version
    fn insert_loop(&mut self) {
        Terminal::set_cursor_style(CursorStyle::SteadyBar).unwrap();
        self.print_bottom("[ insert ]");
        loop {
            self.refresh_screen();
            if let Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) = Terminal::read_event()
            {
                match (code, modifiers) {
                    (KeyCode::Esc, _) => break,
                    (KeyCode::Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                        self.view.insert_char(self.size, c);
                        self.insert_message(&c.to_string());
                    }
                    (KeyCode::Tab, KeyModifiers::NONE) => {
                        self.view.insert_char(self.size, '\t');
                        self.insert_message("Tab");
                    }
                    (KeyCode::Enter, KeyModifiers::NONE) => {
                        self.view.insert_char(self.size, '\n');
                        self.insert_message("Enter");
                    }
                    (KeyCode::Delete, KeyModifiers::NONE) => {
                        self.view.remove_char();
                        self.insert_message("Delete");
                    }
                    (KeyCode::Backspace, KeyModifiers::NONE) => {
                        // just detect if the caret is at the beginning of the buffer
                        // so we don't need to use caret_screen_position() here
                        if self.view.cursor.col_idx() > 0 || self.view.cursor.line_idx() > 0 {
                            self.view.move_position(self.size, KeyCode::Left);
                            self.view.remove_char();
                            self.insert_message("Backspace");
                        }
                    }
                    _ => (),
                }
            }
        }
        Terminal::set_cursor_style(CursorStyle::DefaultUserShape).unwrap();
    }
    fn insert_message(&self, input: &str) {
        let line = self
            .view
            .get_line(self.view.cursor.line_idx())
            .map_or("no line", line::Line::content);

        self.print_bottom(&format!("[ insert ] input: {input}, content: {line}"));
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
        let mut editor = Editor::default();
        assert_eq!(editor.size, Size::default());
        editor.handle_resize_event(10, 10);
        assert_eq!(editor.size, Size::new(10, 10));
    }
}
