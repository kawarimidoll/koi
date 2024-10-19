use super::buffer::Buffer;
use super::cursor::Cursor;
use super::line::Line;
use super::position::Position;
use super::size::Size;
use super::terminal::Terminal;
use super::text_fragment::TextFragment;
use std::{cmp::min, io::Error};

#[derive(Copy, Clone, PartialEq)]
pub enum MoveCode {
    Left,
    Right,
    Up,
    Down,
    FirstLine,
    LastLine,
    FirstChar,
    LastChar,
    FirstNonBlank,
}

#[derive(Copy, Clone, PartialEq)]
pub enum ScrollCode {
    Left(usize),
    Right(usize),
    Up(usize),
    Down(usize),
}

pub struct View {
    pub cursor: Cursor,
    // offset is top-left vertex of the visible buffer
    pub offset: Position,

    size: Size,
    // I don't think View should have buffer as a member, but put it here for now
    pub buffer: Buffer,
}

impl View {
    pub fn new(buffer: Buffer, size: Size) -> Self {
        Self {
            cursor: Cursor::default(),
            offset: Position::default(),
            size,
            buffer,
        }
    }
    pub fn has_path(&self) -> bool {
        self.buffer.has_path()
    }
    pub fn save(&mut self) -> Result<(), Error> {
        self.buffer.save()
    }
    pub fn save_as(&mut self, filename: &str) -> Result<(), Error> {
        self.buffer.save_as(filename)
    }
    pub fn set_size(&mut self, size: Size) {
        self.size = size;
        self.ensure_redraw();
    }
    pub fn caret_screen_position(&self) -> Position {
        self.cursor.get_screen_position(&self.offset)
    }
    // pub fn get_current_line_content(&self) -> String {
    //     self.get_line(self.cursor.line_idx())
    //         .map(|line| line.content().to_string())
    //         .unwrap_or_default()
    // }
    pub fn get_line(&self, line_idx: usize) -> Option<&Line> {
        self.buffer.lines.get(line_idx)
    }
    pub fn get_fragment_by_position(&self, pos: Position) -> Option<&TextFragment> {
        self.get_line(pos.line_idx)
            .and_then(|line| line.get_fragment_by_col_idx(pos.col_idx))
    }

    // TODO: support string
    pub fn insert_char(&mut self, c: char) {
        if c == '\n' {
            if self.buffer.insert_newline(self.cursor.position()) {
                self.ensure_redraw();
                self.move_position(MoveCode::Right);
            }
            return;
        }
        if self.buffer.insert(&c.to_string(), self.cursor.position()) {
            self.ensure_redraw();
            self.move_position(MoveCode::Right);
        }
    }
    pub fn insert_char_without_move(&mut self, c: char) {
        if c == '\n' {
            if self.buffer.insert_newline(self.cursor.position()) {
                self.ensure_redraw();
            }
            return;
        }
        if self.buffer.insert(&c.to_string(), self.cursor.position()) {
            self.ensure_redraw();
        }
    }
    pub fn remove_char(&mut self) {
        if self.buffer.remove_char(self.cursor.position()) {
            self.ensure_redraw();
        }
    }

    pub fn height(&self) -> usize {
        self.size.height
    }
    pub fn scroll_screen(&mut self, code: ScrollCode) {
        let saved_offset = self.offset;
        match code {
            ScrollCode::Left(_step) => self.scroll_left(),
            ScrollCode::Right(_step) => self.scroll_right(),
            ScrollCode::Up(step) => self.scroll_up(step),
            ScrollCode::Down(step) => self.scroll_down(step),
            // ScrollCode::PageUp => self.scroll_up(self.size.height),
            // ScrollCode::PageDown => self.scroll_down(self.size.height),
        };
        self.buffer.needs_redraw = self.offset != saved_offset;
    }
    fn scroll_left(&mut self) {
        self.cursor.move_prev_grapheme_nowrap();
        self.offset.col_idx = self.offset.col_idx.saturating_sub(1);
    }
    fn scroll_right(&mut self) {
        self.cursor.move_next_grapheme_nowrap(&self.buffer);
        self.offset.col_idx = min(
            self.offset.col_idx.saturating_add(1),
            self.buffer
                .get_line_col_width(self.cursor.line_idx())
                .saturating_add(1)
                .saturating_sub(self.size.width),
        );
    }
    fn scroll_up(&mut self, step: usize) {
        let off_l = self.offset.line_idx;
        self.cursor.move_prev_line(step, &self.buffer);
        self.offset.line_idx = off_l.saturating_sub(step);
    }
    fn scroll_down(&mut self, step: usize) {
        let off_l = self.offset.line_idx;
        self.cursor.move_next_line(step, &self.buffer);
        self.offset.line_idx = min(
            off_l.saturating_add(step),
            self.buffer
                .get_lines_count()
                .saturating_add(1)
                .saturating_sub(self.size.height),
        );
    }
    pub fn move_position(&mut self, code: MoveCode) {
        match code {
            MoveCode::Left => self.cursor.move_prev_grapheme(&self.buffer),
            MoveCode::Right => self.cursor.move_next_grapheme(&self.buffer),
            MoveCode::Up => self.cursor.move_prev_line(1, &self.buffer),
            MoveCode::Down => self.cursor.move_next_line(1, &self.buffer),
            MoveCode::FirstChar => self.cursor.move_first_char(&self.buffer),
            MoveCode::LastChar => self.cursor.move_last_char(&self.buffer),
            MoveCode::FirstLine => self.cursor.move_first_line(&self.buffer),
            MoveCode::LastLine => self.cursor.move_last_line(&self.buffer),
            MoveCode::FirstNonBlank => self.cursor.move_first_non_blank(&self.buffer),
        };
        self.scroll_into_view();
    }

    fn scroll_into_view(&mut self) {
        let Position { line_idx, col_idx } = self.cursor.position();
        let Size { width, height } = self.size;
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
    pub fn render(&mut self) -> Result<(), Error> {
        self.buffer
            .render(self.size, self.offset, Terminal::print_row)
    }
    pub fn search(&self, _query: &str) {
        // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll() {
        let buffer = Buffer::from_string("ab\ncd\n");
        let size = Size::new(2, 2);
        let mut view = View::new(buffer, size);
        view.scroll_screen(ScrollCode::Down(1));
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        assert_eq!(view.offset, Position::new(1, 0));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(ScrollCode::Down(1));
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        assert_eq!(view.offset, Position::new(1, 0));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;
        view.scroll_screen(ScrollCode::Up(1));
        assert_eq!(view.caret_screen_position(), Position::new(1, 0));
        assert_eq!(view.offset, Position::new(0, 0));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(ScrollCode::Up(1));
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        assert_eq!(view.offset, Position::new(0, 0));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;

        view.scroll_screen(ScrollCode::Right(1));
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        assert_eq!(view.offset, Position::new(0, 1));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(ScrollCode::Right(1));
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        assert_eq!(view.offset, Position::new(0, 1));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;
        view.scroll_screen(ScrollCode::Left(1));
        assert_eq!(view.caret_screen_position(), Position::new(0, 1));
        assert_eq!(view.offset, Position::new(0, 0));
        assert_eq!(view.buffer.needs_redraw, true);
        view.buffer.needs_redraw = false;
        view.scroll_screen(ScrollCode::Left(1));
        assert_eq!(view.caret_screen_position(), Position::new(0, 0));
        assert_eq!(view.offset, Position::new(0, 0));
        assert_eq!(view.buffer.needs_redraw, false);
        view.buffer.needs_redraw = false;
    }

    #[test]
    fn test_insert_char() {
        let buffer = Buffer::from_string("this\nis\ntest.\n");
        let size = Size::new(10, 10);
        let mut view = View::new(buffer, size);
        view.cursor.set_position(Position::new(0, 1), &view.buffer);
        view.insert_char('o');
        assert_eq!(view.buffer.lines[0].content(), "tohis");
        assert_eq!(view.cursor.position(), Position::new(0, 2));
        view.insert_char('\n');
        assert_eq!(view.buffer.lines[0].content(), "to");
        assert_eq!(view.buffer.lines[1].content(), "his");
        assert_eq!(view.cursor.position(), Position::new(1, 0));
    }

    #[test]
    fn test_remove_char() {
        let buffer = Buffer::from_string("this\nis\ntest.\n");
        let size = Size::new(10, 10);
        let mut view = View::new(buffer, size);
        view.cursor.set_position(Position::new(0, 1), &view.buffer);
        view.remove_char();
        assert_eq!(view.buffer.lines[0].content(), "tis");
        view.cursor.set_position(Position::new(0, 3), &view.buffer);
        view.remove_char();
        assert_eq!(view.buffer.lines[0].content(), "tisis");
    }
}
