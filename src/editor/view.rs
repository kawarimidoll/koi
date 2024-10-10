use super::buffer::Buffer;
use super::line::Line;
use super::position::Position;
use super::size::Size;
use super::terminal::Terminal;
use super::text_fragment::TextFragment;
use crossterm::event::KeyCode::{self, Down, End, Home, Left, PageDown, PageUp, Right, Up};
use std::{cmp::min, io::Error};

#[derive(Default)]
pub struct Cursor {
    position: Position,
    col_want: usize,
}

impl Cursor {
    pub fn set_line_idx(&mut self, line_idx: usize, current_buffer: &Buffer) {
        self.position.line_idx = min(line_idx, current_buffer.get_lines_count());
    }
    pub fn set_col_idx(&mut self, col_idx: usize, current_buffer: &Buffer) {
        self.col_want = col_idx;
        self.snap_col_idx(current_buffer);
    }
    pub fn snap_col_idx(&mut self, current_buffer: &Buffer) {
        if self.position.line_idx >= current_buffer.get_lines_count() {
            // invalid line_idx
            self.position.col_idx = 0;
            return;
        }

        let line = &current_buffer.lines[self.position.line_idx];
        self.position.col_idx = if let Some(fragment) = line.get_fragment_by_col_idx(self.col_want)
        {
            fragment.left_col_width()
        } else {
            line.col_width()
        };
    }
    #[cfg(test)]
    pub fn set_position(&mut self, position: Position, current_buffer: &Buffer) {
        self.set_line_idx(position.line_idx, current_buffer);
        self.set_col_idx(position.col_idx, current_buffer);
    }
    pub fn get_screen_position(&self, offset: &Position) -> Position {
        self.position.saturating_sub(offset)
    }
    pub fn position(&self) -> Position {
        self.position
    }
    pub fn line_idx(&self) -> usize {
        self.position.line_idx
    }
    pub fn col_idx(&self) -> usize {
        self.position.col_idx
    }
    pub fn col_want(&self) -> usize {
        self.col_want
    }
    pub fn move_left_edge(&mut self, current_buffer: &Buffer) {
        self.set_col_idx(0, current_buffer);
    }
    pub fn move_right_edge(&mut self, current_buffer: &Buffer) {
        self.set_col_idx(usize::MAX, current_buffer);
    }
    fn move_prev_grapheme(&mut self, current_buffer: &Buffer) {
        if self.position.col_idx > 0 {
            self.set_col_idx(self.position.col_idx.saturating_sub(1), current_buffer);
        } else if self.position.line_idx > 0 {
            self.move_prev_line(1, current_buffer);
            // shorthand: no need to use set_col_idx
            self.position.col_idx = current_buffer.get_line_col_width(self.position.line_idx);
            self.col_want = self.position.col_idx;
        }
    }
    fn move_next_grapheme(&mut self, current_buffer: &Buffer) {
        if self.position.line_idx >= current_buffer.lines.len() {
            self.set_col_idx(0, current_buffer);
            return;
        }
        let line = &current_buffer.lines[self.position.line_idx];
        let step = if let Some(fragment) = line.get_fragment_by_col_idx(self.position.col_idx) {
            fragment.width()
        } else {
            1
        };

        if self.position.col_idx < current_buffer.get_line_col_width(self.position.line_idx) {
            // shorthand: no need to use set_col_idx
            self.position.col_idx = self.position.col_idx.saturating_add(step);
            self.col_want = self.position.col_idx;
        } else if self.position.line_idx < current_buffer.get_lines_count() {
            self.move_next_line(1, current_buffer);
            // shorthand: no need to use set_col_idx
            self.position.col_idx = 0;
            self.col_want = self.position.col_idx;
        }
    }
    fn move_prev_grapheme_nowrap(&mut self) {
        // shorthand: no need to use set_col_idx
        self.position.col_idx = self.position.col_idx.saturating_sub(1);
        self.col_want = self.position.col_idx;
    }
    fn move_next_grapheme_nowrap(&mut self, current_buffer: &Buffer) {
        // shorthand: no need to use set_col_idx
        self.position.col_idx = min(
            self.position.col_idx.saturating_add(1),
            current_buffer.get_line_col_width(self.position.line_idx),
        );
        self.col_want = self.position.col_idx;
    }
    fn move_prev_line(&mut self, step: usize, current_buffer: &Buffer) {
        // shorthand: no need to use set_line_idx
        self.position.line_idx = self.position.line_idx.saturating_sub(step);
        self.snap_col_idx(current_buffer);
    }
    fn move_next_line(&mut self, step: usize, current_buffer: &Buffer) {
        self.set_line_idx(self.position.line_idx.saturating_add(step), current_buffer);
        self.snap_col_idx(current_buffer);
    }
}

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
                self.move_position(size, Right);
            }
            return;
        }
        if self.buffer.insert(&c.to_string(), self.cursor.position()) {
            self.ensure_redraw();
            self.move_position(size, Right);
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
            Left => self.scroll_left(),
            Right => self.scroll_right(size),
            Up => self.scroll_up(1),
            Down => self.scroll_down(size, 1),
            PageUp => self.scroll_up(size.height),
            PageDown => self.scroll_down(size, size.height),
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
            Left => self.cursor.move_prev_grapheme(&self.buffer),
            Right => self.cursor.move_next_grapheme(&self.buffer),
            Up => self.cursor.move_prev_line(1, &self.buffer),
            Down => self.cursor.move_next_line(1, &self.buffer),
            Home => self.cursor.move_left_edge(&self.buffer),
            End => self.cursor.move_right_edge(&self.buffer),
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
    fn test_move_prev_grapheme() {
        let mut view = View::default();
        let size = Size::new(10, 10);

        view.buffer.lines = Buffer::gen_lines("a\nb\n");
        view.cursor.set_position(Position::new(1, 1), &view.buffer);
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0)); // wrap
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0)); // boundary

        // test for full-width character
        view.buffer.lines = Buffer::gen_lines("aあbい\nうcえd\n");
        view.cursor.set_position(Position::new(1, 6), &view.buffer);
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(1, 5));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(1, 3));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(1, 2));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 6));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 4));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 3));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
    }

    #[test]
    fn test_move_next_grapheme() {
        let mut view = View::default();
        let size = Size::new(10, 10);
        view.buffer.lines = Buffer::gen_lines("a\nb\n");
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(1, 1)); // wrap
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(2, 0));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(2, 0)); // boundary

        // test for full-width character
        view.buffer.lines = Buffer::gen_lines("aあbい\nうcえd\n");
        view.cursor.set_position(Position::default(), &view.buffer);
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 3));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 4));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 6));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(1, 2));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(1, 3));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(1, 5));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(1, 6));
    }

    #[test]
    fn test_move_prev_line() {
        let mut view = View::default();
        let size = Size::new(10, 10);
        view.buffer.lines = Buffer::gen_lines("this\nis\ntest.\n");
        view.cursor.set_position(Position::new(2, 3), &view.buffer);
        view.move_position(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(1, 2)); // snap
        view.move_position(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(0, 3));
        view.move_position(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(0, 3)); // boundary

        // test for full-width character
        view.buffer.lines = Buffer::gen_lines("aあ\nいb\ncう\nえe\n");
        view.cursor.set_position(Position::new(3, 2), &view.buffer);
        view.move_position(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(2, 1));
        view.move_position(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(1, 2));
        view.move_position(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
    }

    #[test]
    fn test_move_next_line() {
        let mut view = View::default();
        let size = Size::new(10, 10);
        view.buffer.lines = Buffer::gen_lines("this\nis\ntest.\n");
        view.cursor.set_position(Position::new(0, 3), &view.buffer);
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(1, 2)); // snap
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(2, 3));
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(3, 0));
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(3, 0)); // boundary

        // test for full-width character
        view.buffer.lines = Buffer::gen_lines("aあ\nいb\ncう\nえe\n");
        view.cursor.set_position(Position::new(0, 1), &view.buffer);
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(2, 1));
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(3, 0));
    }

    #[test]
    fn test_scroll() {
        let mut view = View::default();
        let size = Size::new(2, 2);
        view.buffer.lines = Buffer::gen_lines("ab\ncd\n");
        view.scroll_screen(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        assert_eq!(view.offset, Position::new(1, 0));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        assert_eq!(view.offset, Position::new(1, 0));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        assert_eq!(view.offset, Position::new(0, 0));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        assert_eq!(view.offset, Position::new(0, 0));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;

        view.scroll_screen(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        assert_eq!(view.offset, Position::new(0, 1));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        assert_eq!(view.offset, Position::new(0, 1));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        assert_eq!(view.offset, Position::new(0, 0));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, Left);
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
