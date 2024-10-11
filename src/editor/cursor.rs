use super::buffer::Buffer;
use super::position::Position;
use super::text_fragment::TextFragment;
use std::{cmp::min, fmt};

#[derive(Default)]
pub struct Cursor {
    line_idx: usize,
    col_idx: usize,
    col_want: usize,
}

impl Cursor {
    // TODO consider screen size
    pub fn set_line_idx(&mut self, line_idx: usize, current_buffer: &Buffer) {
        self.line_idx = min(line_idx, current_buffer.get_lines_count());
    }
    pub fn set_col_idx(&mut self, col_idx: usize, current_buffer: &Buffer) {
        self.col_want = col_idx;
        self.snap_col_idx(current_buffer);
    }
    pub fn snap_col_idx(&mut self, current_buffer: &Buffer) {
        if self.line_idx >= current_buffer.get_lines_count() {
            // invalid line_idx
            self.col_idx = 0;
            return;
        }

        let line = &current_buffer.lines[self.line_idx];

        self.col_idx = line
            .get_fragment_by_col_idx(self.col_want)
            .map_or_else(|| line.col_width(), TextFragment::left_col_width);
    }
    #[cfg(test)]
    pub fn set_position(&mut self, position: Position, current_buffer: &Buffer) {
        self.set_line_idx(position.line_idx, current_buffer);
        self.set_col_idx(position.col_idx, current_buffer);
    }
    pub fn get_screen_position(&self, offset: &Position) -> Position {
        self.position().saturating_sub(offset)
    }
    pub fn position(&self) -> Position {
        Position {
            line_idx: self.line_idx,
            col_idx: self.col_idx,
        }
    }
    pub fn line_idx(&self) -> usize {
        self.line_idx
    }
    pub fn col_idx(&self) -> usize {
        self.col_idx
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
    pub fn move_prev_grapheme(&mut self, current_buffer: &Buffer) {
        if self.col_idx > 0 {
            self.set_col_idx(self.col_idx.saturating_sub(1), current_buffer);
        } else if self.line_idx > 0 {
            self.move_prev_line(1, current_buffer);
            // shorthand: no need to use set_col_idx
            self.col_idx = current_buffer.get_line_col_width(self.line_idx);
            self.col_want = self.col_idx;
        }
    }
    pub fn move_next_grapheme(&mut self, current_buffer: &Buffer) {
        if self.line_idx >= current_buffer.get_lines_count() {
            self.set_col_idx(0, current_buffer);
            return;
        }
        let line = &current_buffer.lines[self.line_idx];
        let step = line
            .get_fragment_by_col_idx(self.col_idx)
            .map_or(1, TextFragment::width);

        if self.col_idx < current_buffer.get_line_col_width(self.line_idx) {
            // shorthand: no need to use set_col_idx
            self.col_idx = self.col_idx.saturating_add(step);
            self.col_want = self.col_idx;
        } else if self.line_idx < current_buffer.get_lines_count() {
            self.move_next_line(1, current_buffer);
            // shorthand: no need to use set_col_idx
            self.col_idx = 0;
            self.col_want = self.col_idx;
        }
    }
    pub fn move_prev_grapheme_nowrap(&mut self) {
        // shorthand: no need to use set_col_idx
        self.col_idx = self.col_idx.saturating_sub(1);
        self.col_want = self.col_idx;
    }
    pub fn move_next_grapheme_nowrap(&mut self, current_buffer: &Buffer) {
        // shorthand: no need to use set_col_idx
        self.col_idx = min(
            self.col_idx.saturating_add(1),
            current_buffer.get_line_col_width(self.line_idx),
        );
        self.col_want = self.col_idx;
    }
    pub fn move_prev_line(&mut self, step: usize, current_buffer: &Buffer) {
        // shorthand: no need to use set_line_idx
        self.line_idx = self.line_idx.saturating_sub(step);
        self.snap_col_idx(current_buffer);
    }
    pub fn move_next_line(&mut self, step: usize, current_buffer: &Buffer) {
        self.set_line_idx(self.line_idx.saturating_add(step), current_buffer);
        self.snap_col_idx(current_buffer);
    }
}

impl fmt::Display for Cursor {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "({}, {}, {})",
            self.line_idx(),
            self.col_idx(),
            self.col_want()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_prev_grapheme() {
        let buffer = Buffer::from_string("a\nb\n");
        let mut cursor = Cursor::default();
        cursor.set_position(Position::new(1, 1), &buffer);
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(1, 0)); // wrap
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 1));
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 0));
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 0)); // boundary

        // test for full-width character
        let buffer = Buffer::from_string("aあbい\nうcえd\n");
        cursor.set_position(Position::new(1, 6), &buffer);
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(1, 5));
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(1, 3));
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(1, 2));
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(1, 0));
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 6));
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 4));
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 3));
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 1));
        cursor.move_prev_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 0));
    }

    #[test]
    fn test_move_next_grapheme() {
        let buffer = Buffer::from_string("a\nb\n");
        let mut cursor = Cursor::default();
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 1));
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(1, 0));
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(1, 1)); // wrap
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(2, 0));
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(2, 0)); // boundary

        // test for full-width character
        let buffer = Buffer::from_string("aあbい\nうcえd\n");
        cursor.set_position(Position::default(), &buffer);
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 1));
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 3));
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 4));
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(0, 6));
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(1, 0));
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(1, 2));
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(1, 3));
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(1, 5));
        cursor.move_next_grapheme(&buffer);
        assert_eq!(cursor.position(), Position::new(1, 6));
    }

    #[test]
    fn test_move_prev_line() {
        let buffer = Buffer::from_string("this\nis\ntest.\n");
        let mut cursor = Cursor::default();
        cursor.set_position(Position::new(2, 3), &buffer);
        cursor.move_prev_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(1, 2)); // snap
        cursor.move_prev_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(0, 3));
        cursor.move_prev_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(0, 3)); // boundary

        // test for full-width character
        let buffer = Buffer::from_string("aあ\nいb\ncう\nえe\n");
        cursor.set_position(Position::new(3, 2), &buffer);
        cursor.move_prev_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(2, 1));
        cursor.move_prev_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(1, 2));
        cursor.move_prev_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(0, 1));
    }

    #[test]
    fn test_move_next_line() {
        let buffer = Buffer::from_string("this\nis\ntest.\n");
        let mut cursor = Cursor::default();
        cursor.set_position(Position::new(0, 3), &buffer);
        cursor.move_next_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(1, 2)); // snap
        cursor.move_next_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(2, 3));
        cursor.move_next_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(3, 0));
        cursor.move_next_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(3, 0)); // boundary

        // test for full-width character
        let buffer = Buffer::from_string("aあ\nいb\ncう\nえe\n");
        cursor.set_position(Position::new(0, 1), &buffer);
        cursor.move_next_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(1, 0));
        cursor.move_next_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(2, 1));
        cursor.move_next_line(1, &buffer);
        assert_eq!(cursor.position(), Position::new(3, 0));
    }
}
