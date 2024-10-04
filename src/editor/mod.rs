// 日本語🇯🇵の表示テスト
use crossterm::event::{
    read,
    Event::{Key, Resize},
    KeyCode::{Char, Down, End, Home, Left, PageDown, PageUp, Right, Up},
    KeyEvent, KeyModifiers,
};
use std::io::Error;
use terminal::{Position, Size, Terminal};
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
        let bottom_line = self.size.height.saturating_sub(1);
        Terminal::print_row(bottom_line, "Type something. Press 'q' to quit.").unwrap();
        self.move_caret();

        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            match read() {
                Ok(Key(key_event)) => {
                    // necessary for windows
                    if key_event.kind == crossterm::event::KeyEventKind::Press {
                        self.handle_key_event(key_event);
                    }
                }
                Ok(Resize(width16, height16)) => {
                    self.handle_resize_event(width16, height16);
                }
                Err(err) => {
                    Terminal::print_row(bottom_line, &format!("{err}")).unwrap();
                }
                _ => {
                    Terminal::print_row(bottom_line, "Unsupported event!").unwrap();
                }
            }
        }
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        let height = self.size.height;
        let KeyEvent {
            code, modifiers, ..
        } = event;

        match (code, modifiers) {
            (Char('q'), KeyModifiers::NONE) => self.should_quit = true,

            (Left | Down | Right | Up, KeyModifiers::SHIFT)
            | (PageDown | PageUp, KeyModifiers::NONE) => {
                self.view.scroll_screen(self.size, code);
            }
            (Left | Down | Right | Up | Home | End, KeyModifiers::NONE) => {
                self.view.move_position(self.size, code);
            }
            _ => (),
        }
        let Position {
            col: doc_x,
            row: doc_y,
        } = self.view.position;
        let Position { col, row } = self.view.caret_screen_position(&self.view.buffer.lines);
        let Position {
            col: off_c,
            row: off_r,
        } = self.view.offset;

        let info = if let Some(line) = self.view.buffer.lines.get(self.view.position.row) {
            if let Some(fragment) = line.get_fragment_by_col_idx(self.view.position.col) {
                &format!(
                    "{}, {}, {}",
                    fragment.grapheme,
                    fragment.width(),
                    fragment.left_col_width
                )
            } else {
                ""
            }
        } else {
            ""
        };

        let _ = Terminal::print_row(
            height.saturating_sub(1),
            &format!("loc: {doc_x},{doc_y}, pos: {col},{row}, off: {off_c},{off_r}, [{info}]"),
        );
    }
    #[allow(clippy::as_conversions)]
    fn handle_resize_event(&mut self, width16: u16, height16: u16) {
        let width = width16 as usize;
        let height = height16 as usize;
        // let _ = Terminal::print_row(height - 1, &format!("Resize to: {width:?}, {height:?}"));
        self.size = Size { width, height };
        self.view.buffer.ensure_redraw();
    }
    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();
        let _ = self
            .view
            .buffer
            .render(self.size, self.view.offset, Terminal::print_row);
        self.move_caret();
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
    }
    fn move_caret(&self) {
        Terminal::move_caret_to(self.view.caret_screen_position(&self.view.buffer.lines)).unwrap();
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
