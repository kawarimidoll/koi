use super::buffer::Buffer;
use super::terminal::{Position, Size, Terminal};
use super::text_fragment::TextFragment;
use crossterm::event::KeyCode::{self, Down, End, Home, Left, PageDown, PageUp, Right, Up};
use std::{cmp::min, io::Error};

#[derive(Default)]
pub struct View {
    // position is col-row vertex of the cursor from the document origin
    pub position: Position,
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
        self.caret_snap_on_line().saturating_sub(&self.offset)
    }
    fn caret_snap_on_line(&self) -> Position {
        let col = if let Some(line) = self.buffer.lines.get(self.position.row) {
            if let Some(fragment) = line.get_fragment_by_col_idx(self.position.col) {
                fragment.left_col_width()
            } else {
                line.col_width()
            }
        } else {
            0
        };

        Position {
            col,
            row: min(self.position.row, self.buffer.lines.len()),
        }
    }
    pub fn get_fragment_by_position(&self, pos: Position) -> Option<&TextFragment> {
        self.buffer
            .lines
            .get(pos.row)
            .and_then(|line| line.get_fragment_by_col_idx(pos.col))
    }

    // TODO: support string
    pub fn insert_char(&mut self, size: Size, c: char) {
        if self.buffer.insert(&c.to_string(), self.position) {
            self.ensure_redraw();
            self.move_position(size, Right);
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
        self.move_prev_grapheme_nowrap();
        self.offset.col = self.offset.col.saturating_sub(1);
    }
    fn scroll_right(&mut self, size: Size) {
        self.move_next_grapheme_nowrap();
        self.offset.col = min(
            self.offset.col.saturating_add(1),
            self.buffer
                .get_line_col_width(self.position.row)
                .saturating_add(1)
                .saturating_sub(size.width),
        );
    }
    fn scroll_up(&mut self, step: usize) {
        let off_r = self.offset.row;
        self.move_prev_line(step);
        self.offset.row = off_r.saturating_sub(step);
    }
    fn scroll_down(&mut self, size: Size, step: usize) {
        let off_r = self.offset.row;
        self.move_next_line(step);
        self.offset.row = min(
            off_r.saturating_add(step),
            self.buffer
                .get_lines_count()
                .saturating_add(1)
                .saturating_sub(size.height),
        );
    }
    pub fn move_position(&mut self, size: Size, code: KeyCode) {
        match code {
            Left => self.move_prev_grapheme(),
            Right => self.move_next_grapheme(),
            Up => self.move_prev_line(1),
            Down => self.move_next_line(1),
            Home => self.position.col = 0,
            End => self.position.col = usize::MAX,
            _ => (),
        };
        self.scroll_into_view(size);
    }
    #[allow(clippy::arithmetic_side_effects)]
    // allow this because boundary condition is confirmed by myself
    fn move_prev_grapheme(&mut self) {
        self.position.col = self.caret_snap_on_line().col;
        if self.position.col > 0 {
            self.position.col -= 1;
            self.position.col = self.caret_snap_on_line().col;
        } else if self.position.row > 0 {
            self.move_prev_line(1);
            self.position.col = self.buffer.get_line_col_width(self.position.row);
        }
    }
    fn move_next_grapheme(&mut self) {
        self.position.col = self.caret_snap_on_line().col;

        let step = if let Some(line) = self.buffer.lines.get(self.position.row) {
            if let Some(fragment) = line.get_fragment_by_col_idx(self.position.col) {
                fragment.width()
            } else {
                1
            }
        } else {
            0
        };

        if self.position.col < self.buffer.get_line_col_width(self.position.row) {
            self.position.col = self.position.col.saturating_add(step);
        } else if self.position.row < self.buffer.get_lines_count() {
            self.move_next_line(1);
            self.position.col = 0;
        }
    }
    fn move_prev_grapheme_nowrap(&mut self) {
        self.position.col = self.position.col.saturating_sub(1);
    }
    fn move_next_grapheme_nowrap(&mut self) {
        self.position.col = min(
            self.position.col.saturating_add(1),
            self.buffer.get_line_col_width(self.position.row),
        );
    }
    fn move_prev_line(&mut self, step: usize) {
        if self.position.row > 0 {
            self.position.row = self.position.row.saturating_sub(step);
        }
    }
    fn move_next_line(&mut self, step: usize) {
        if self.position.row < self.buffer.get_lines_count() {
            self.position.row = min(
                self.position.row.saturating_add(step),
                self.buffer.get_lines_count(),
            );
        }
    }
    fn scroll_into_view(&mut self, size: Size) {
        let Position { col, row } = self.caret_snap_on_line();
        let Size { width, height } = size;
        // horizontal
        if col < self.offset.col {
            self.offset.col = col;
            self.buffer.ensure_redraw();
        } else if col >= self.offset.col.saturating_add(width) {
            self.offset.col = col.saturating_add(1).saturating_sub(width);
            self.buffer.ensure_redraw();
        }
        // vertical
        if row < self.offset.row {
            self.offset.row = row;
            self.buffer.ensure_redraw();
        } else if row >= self.offset.row.saturating_add(height) {
            self.offset.row = row.saturating_add(1).saturating_sub(height);
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
        view.position = Position::new(1, 1);
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1)); // wrap
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0)); // boundary

        // test for full-width character
        view.buffer.lines = Buffer::gen_lines("aあbい\nうcえd\n");
        view.position = Position::new(6, 1);
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(5, 1));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(3, 1));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(2, 1));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(6, 0));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(4, 0));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(3, 0));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        view.move_position(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
    }

    #[test]
    fn test_move_next_grapheme() {
        let mut view = View::default();
        let size = Size::new(10, 10);
        view.buffer.lines = Buffer::gen_lines("a\nb\n");
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(1, 1)); // wrap
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 2));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 2)); // boundary

        // test for full-width character
        view.buffer.lines = Buffer::gen_lines("aあbい\nうcえd\n");
        view.position = Position::default();
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(3, 0));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(4, 0));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(6, 0));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(2, 1));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(3, 1));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(5, 1));
        view.move_position(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(6, 1));
    }

    #[test]
    fn test_move_prev_line() {
        let mut view = View::default();
        let size = Size::new(10, 10);
        view.buffer.lines = Buffer::gen_lines("this\nis\ntest.\n");
        view.position = Position::new(3, 2);
        view.move_position(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(2, 1)); // snap
        view.move_position(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(3, 0));
        view.move_position(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(3, 0)); // boundary

        // test for full-width character
        view.buffer.lines = Buffer::gen_lines("aあ\nいb\ncう\nえe\n");
        view.position = Position::new(2, 3);
        view.move_position(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(1, 2));
        view.move_position(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(2, 1));
        view.move_position(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
    }

    #[test]
    fn test_move_next_line() {
        let mut view = View::default();
        let size = Size::new(10, 10);
        view.buffer.lines = Buffer::gen_lines("this\nis\ntest.\n");
        view.position = Position::new(3, 0);
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(2, 1)); // snap
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(3, 2));
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(0, 3));
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(0, 3)); // boundary

        // test for full-width character
        view.buffer.lines = Buffer::gen_lines("aあ\nいb\ncう\nえe\n");
        view.position = Position::new(1, 0);
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(1, 2));
        view.move_position(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(0, 3));
    }

    #[test]
    fn test_scroll() {
        let mut view = View::default();
        let size = Size::new(2, 2);
        view.buffer.lines = Buffer::gen_lines("ab\ncd\n");
        view.scroll_screen(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        assert_eq!(view.offset, Position::new(0, 1));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, Down);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        assert_eq!(view.offset, Position::new(0, 1));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, Up);
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
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
        assert_eq!(view.offset, Position::new(1, 0));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, Right);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        assert_eq!(view.offset, Position::new(1, 0));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;
        view.scroll_screen(size, Left);
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
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
        view.position = Position { col: 1, row: 0 };
        view.insert_char(size, 'o');
        assert_eq!(view.buffer.lines[0].content(), "tohis");
        assert_eq!(view.position, Position { col: 2, row: 0 });
    }
}
