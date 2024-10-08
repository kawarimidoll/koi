use super::buffer::Buffer;
use super::cursor::Cursor;
use super::line::Line;
use super::position::Position;
use super::size::Size;
use super::text_fragment::TextFragment;
use super::terminal::{KeyCode, Terminal};
use std::{cmp::min, io::Error};

#[derive(Default)]
pub struct View {
    pub cursor: Cursor,
    // offset is top-left vertex of the visible buffer
    pub offset: Position,

    // I don't think View should have buffer as a member, but put it here for now
    pub buffer: Buffer,
}

impl View {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            ..Self::default()
        }
    }
    pub fn caret_screen_position(&self) -> Position {
        self.cursor.get_screen_position(&self.offset)
    }
    pub fn get_line(&self, line_idx: usize) -> Option<&Line> {
        self.buffer.lines.get(line_idx)
    }
    pub fn get_fragment_by_position(&self, pos: Position) -> Option<&TextFragment> {
        self.buffer
            .lines
            .get(pos.line_idx)
            .and_then(|line| line.get_fragment_by_col_idx(pos.col_idx))
    }

    // TODO: support string
    pub fn insert_char(&mut self, size: Size, c: char) {
        if c == '\n' {
            if self.buffer.insert_newline(self.cursor.position()) {
                self.ensure_redraw();
                self.move_position(size, KeyCode::Right);
            }
            return;
        }
        if self.buffer.insert(&c.to_string(), self.cursor.position()) {
            self.ensure_redraw();
            self.move_position(size, KeyCode::Right);
        }
    }
    pub fn remove_char(&mut self) {
        if self.buffer.remove_char(self.cursor.position()) {
            self.ensure_redraw();
        }
    }

    pub fn scroll_screen(&mut self, size: Size, code: KeyCode) {
        let saved_offset = self.offset;
        match code {
            KeyCode::Left => self.scroll_left(),
            KeyCode::Right => self.scroll_right(size),
            KeyCode::Up => self.scroll_up(1),
            KeyCode::Down => self.scroll_down(size, 1),
            KeyCode::PageUp => self.scroll_up(size.height),
            KeyCode::PageDown => self.scroll_down(size, size.height),
            _ => (),
        };
        self.buffer.needs_redraw = self.offset != saved_offset;
    }
    fn scroll_left(&mut self) {
        self.cursor.move_prev_grapheme_nowrap();
        self.offset.col_idx = self.offset.col_idx.saturating_sub(1);
    }
    fn scroll_right(&mut self, size: Size) {
        self.cursor.move_next_grapheme_nowrap(&self.buffer);
        self.offset.col_idx = min(
            self.offset.col_idx.saturating_add(1),
            self.buffer
                .get_line_col_width(self.cursor.line_idx())
                .saturating_add(1)
                .saturating_sub(size.width),
        );
    }
    fn scroll_up(&mut self, step: usize) {
        let off_l = self.offset.line_idx;
        self.cursor.move_prev_line(step, &self.buffer);
        self.offset.line_idx = off_l.saturating_sub(step);
    }
    fn scroll_down(&mut self, size: Size, step: usize) {
        let off_l = self.offset.line_idx;
        self.cursor.move_next_line(step, &self.buffer);
        self.offset.line_idx = min(
            off_l.saturating_add(step),
            self.buffer
                .get_lines_count()
                .saturating_add(1)
                .saturating_sub(size.height),
        );
    }
    pub fn move_position(&mut self, size: Size, code: KeyCode) {
        match code {
            KeyCode::Left => self.cursor.move_prev_grapheme(&self.buffer),
            KeyCode::Right => self.cursor.move_next_grapheme(&self.buffer),
            KeyCode::Up => self.cursor.move_prev_line(1, &self.buffer),
            KeyCode::Down => self.cursor.move_next_line(1, &self.buffer),
            KeyCode::Home => self.cursor.move_left_edge(&self.buffer),
            KeyCode::End => self.cursor.move_right_edge(&self.buffer),
            _ => (),
        };
        self.scroll_into_view(size);
    }

    fn scroll_into_view(&mut self, size: Size) {
        let Position { line_idx, col_idx } = self.cursor.position();
        let Size { width, height } = size;
        // horizontal
        if col_idx < self.offset.col_idx {
            self.offset.col_idx = col_idx;
            self.buffer.ensure_redraw();
        } else if col_idx >= self.offset.col_idx.saturating_add(width) {
            self.offset.col_idx = col_idx.saturating_add(1).saturating_sub(width);
            self.buffer.ensure_redraw();
        }
        // vertical
        if line_idx < self.offset.line_idx {
            self.offset.line_idx = line_idx;
            self.buffer.ensure_redraw();
        } else if line_idx >= self.offset.line_idx.saturating_add(height) {
            self.offset.line_idx = line_idx.saturating_add(1).saturating_sub(height);
            self.buffer.ensure_redraw();
        }
    }
    pub fn ensure_redraw(&mut self) {
        self.buffer.needs_redraw = true;
    }
    pub fn render(&mut self, screen_size: Size) -> Result<(), Error> {
        self.buffer
            .render(screen_size, self.offset, Terminal::print_row)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll() {
        let mut view = View::default();
        let size = Size::new(2, 2);
        view.buffer.lines = Buffer::gen_lines("ab\ncd\n");
        view.scroll_screen(size, KeyCode::Down);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        assert_eq!(view.offset, Position::new(1, 0));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, KeyCode::Down);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        assert_eq!(view.offset, Position::new(1, 0));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, KeyCode::Up);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        assert_eq!(view.offset, Position::new(0, 0));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, KeyCode::Up);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        assert_eq!(view.offset, Position::new(0, 0));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;

        view.scroll_screen(size, KeyCode::Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        assert_eq!(view.offset, Position::new(0, 1));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, KeyCode::Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        assert_eq!(view.offset, Position::new(0, 1));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, KeyCode::Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        assert_eq!(view.offset, Position::new(0, 0));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, KeyCode::Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        assert_eq!(view.offset, Position::new(0, 0));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;
    }

    #[test]
    fn test_insert_char() {
        let mut view = View::default();
        let size = Size::new(10, 10);
        view.buffer.lines = Buffer::gen_lines("this\nis\ntest.\n");
        view.cursor.set_position(Position::new(0, 1), &view.buffer);
        view.insert_char(size, 'o');
        assert_eq!(view.buffer.lines[0].content(), "tohis");
        assert_eq!(view.cursor.position(), Position::new(0, 2));
        view.insert_char(size, '\n');
        assert_eq!(view.buffer.lines[0].content(), "to");
        assert_eq!(view.buffer.lines[1].content(), "his");
        assert_eq!(view.cursor.position(), Position::new(1, 0));
    }

    #[test]
    fn test_remove_char() {
        let mut view = View::default();
        view.buffer.lines = Buffer::gen_lines("this\nis\ntest.\n");
        view.cursor.set_position(Position::new(0, 1), &view.buffer);
        view.remove_char();
        assert_eq!(view.buffer.lines[0].content(), "tis");
        view.cursor.set_position(Position::new(0, 3), &view.buffer);
        view.remove_char();
        assert_eq!(view.buffer.lines[0].content(), "tisis");
    }
}
