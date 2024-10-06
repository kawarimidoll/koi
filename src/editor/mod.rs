// 日本語🇯🇵の表示テスト
use crossterm::event::{
    read,
    Event::{Key, Resize},
    KeyCode::{self, Char, Down, End, Esc, Home, Left, PageDown, PageUp, Right, Tab, Up},
    KeyEvent, KeyModifiers,
};
use std::io::Error;
use terminal::{CursorStyle, Size, Terminal};
mod terminal;
// use buffer::Buffer;
mod buffer;
use view::View;
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
            match read() {
                Ok(Key(KeyEvent {
                    code,
                    modifiers,
                    // necessary for windows
                    kind: crossterm::event::KeyEventKind::Press,
                    ..
                })) => {
                    self.handle_key_event(code, modifiers);
                    self.print_bottom(&format!(
                        "loc: {}, pos: {}, off: {}, [{}], key: {}",
                        self.view.position,
                        self.view.caret_screen_position(),
                        self.view.offset,
                        self.view
                            .get_fragment_by_position(self.view.position)
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
                Ok(Resize(width16, height16)) => {
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
            (Char('q'), KeyModifiers::NONE) => self.should_quit = true,
            (Char('i'), KeyModifiers::NONE) => self.insert_loop(),
            (Char('a'), KeyModifiers::NONE) => {
                self.view.move_position(self.size, Right);
                self.insert_loop();
            }

            (Left | Down | Right | Up, KeyModifiers::SHIFT)
            | (PageDown | PageUp, KeyModifiers::NONE) => {
                self.view.scroll_screen(self.size, code);
            }
            (Left | Down | Right | Up | Home | End, KeyModifiers::NONE) => {
                self.view.move_position(self.size, code);
            }
            (Char('h'), KeyModifiers::NONE) => self.view.move_position(self.size, Left),
            (Char('H'), KeyModifiers::SHIFT) => self.view.move_position(self.size, Home),
            (Char('j'), KeyModifiers::NONE) => self.view.move_position(self.size, Down),
            (Char('k'), KeyModifiers::NONE) => self.view.move_position(self.size, Up),
            (Char('l'), KeyModifiers::NONE) => self.view.move_position(self.size, Right),
            (Char('L'), KeyModifiers::SHIFT) => self.view.move_position(self.size, End),
            (Char('f'), KeyModifiers::CONTROL) => self.view.scroll_screen(self.size, PageDown),
            (Char('b'), KeyModifiers::CONTROL) => self.view.scroll_screen(self.size, PageUp),
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
            if let Ok(Key(KeyEvent {
                code,
                modifiers,
                kind: crossterm::event::KeyEventKind::Press,
                ..
            })) = read()
            {
                match (code, modifiers) {
                    (Esc, _) => break,
                    (Char(c), KeyModifiers::NONE | KeyModifiers::SHIFT) => {
                        self.view.insert_char(self.size, c);
                        self.print_bottom(&format!("[ insert ] input: {c}"));
                    (Tab, KeyModifiers::NONE) => {
                        let c = '\t';
                        self.view.insert_char(self.size, c);
                        let line = if let Some(l) = self.view.get_line(self.view.position.row) {
                            l.content()
                        } else {
                            "no line"
                        };
                        self.print_bottom(&format!("[ insert ] input: {c}, content: {line}"));
                    }
                    _ => (),
                }
            }
        }
        Terminal::set_cursor_style(CursorStyle::DefaultUserShape).unwrap();
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
