use super::line::Line;
use super::terminal::{Position, Size};
use crossterm::event::KeyCode::{self, Down, End, Home, Left, PageDown, PageUp, Right, Up};
use std::{cmp::min, fs::read_to_string, io::Error};

#[derive(Default)]
pub struct Buffer {
    // position is col-row vertex from the document origin
    pub position: Position,
    pub render_offset: Position,
    pub lines: Vec<Line>,
    needs_redraw: bool,
}

impl Buffer {
    pub fn new() -> Self {
        let mut buffer = Self::default();
        buffer.handle_args();
        buffer.ensure_redraw();
        buffer
    }
    fn handle_args(&mut self) {
        let args: Vec<String> = std::env::args().collect();
        // only load the first file for now
        if let Some(first) = args.get(1) {
            if let Ok(lines) = Self::load(first) {
                self.lines = lines;
            }
        }
    }
    pub fn load(filename: &str) -> Result<Vec<Line>, Error> {
        let contents = read_to_string(filename)?;
        Ok(Self::gen_lines(&contents))
    }
    fn gen_lines(src: &str) -> Vec<Line> {
        src.lines().map(Line::from).collect()
    }
    pub fn ensure_redraw(&mut self) {
        self.needs_redraw = true;
    }
    pub fn render<F: Fn(usize, &str) -> Result<(), Error>>(
        &mut self,
        size: Size,
        renderer: F,
    ) -> Result<(), Error> {
        // render function
        if !self.needs_redraw {
            return Ok(());
        }
        let Size { width, height } = size;
        let top = self.render_offset.row;
        let left = self.render_offset.col;
        let right = left.saturating_add(width);
        for current_row in 0..height.saturating_sub(1) {
            let current_line = top.saturating_add(current_row);
            if let Some(line) = self.lines.get(current_line) {
                let end = min(right, line.col_width());
                let str = line.get_str_by_col_range(left..end);
                renderer(current_row, &str)?;
                continue;
            }
            renderer(current_row, "~")?;
        }
        // the bottom line is reserved for messages
        self.needs_redraw = false;
        Ok(())
    }
    pub fn caret_screen_position(&self) -> Position {
        self.caret_snap_on_line(&self.lines)
            .saturating_sub(&self.render_offset)
    }
    fn caret_snap_on_line(&self, lines: &[Line]) -> Position {
        let col = if let Some(line) = lines.get(self.position.row) {
            if let Some(fragment) = line.get_fragment_by_col_idx(self.position.col) {
                fragment.left_col_width
            } else {
                line.col_width()
            }
        } else {
            0
        };

        Position {
            col,
            row: min(self.position.row, self.get_lines_count()),
        }
    }
    pub fn scroll_screen(&mut self, size: Size, code: KeyCode) {
        let saved_offset = self.render_offset;
        match code {
            Left => self.scroll_left(),
            Right => self.scroll_right(size),
            Up => self.scroll_up(1),
            Down => self.scroll_down(size, 1),
            PageUp => self.scroll_up(size.height),
            PageDown => self.scroll_down(size, size.height),
            _ => (),
        };
        self.needs_redraw = self.render_offset != saved_offset;
    }
    fn scroll_left(&mut self) {
        self.move_prev_grapheme_nowrap();
        self.render_offset.col = self.render_offset.col.saturating_sub(1);
    }
    fn scroll_right(&mut self, size: Size) {
        self.move_next_grapheme_nowrap();
        self.render_offset.col = min(
            self.render_offset.col.saturating_add(1),
            self.get_current_line_col_width()
                .saturating_add(1)
                .saturating_sub(size.width),
        );
    }
    fn scroll_up(&mut self, step: usize) {
        let off_r = self.render_offset.row;
        self.move_prev_line(step);
        self.render_offset.row = off_r.saturating_sub(step);
    }
    fn scroll_down(&mut self, size: Size, step: usize) {
        let off_r = self.render_offset.row;
        self.move_next_line(step);
        self.render_offset.row = min(
            off_r.saturating_add(step),
            self.get_lines_count()
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
    fn get_current_line_col_width(&self) -> usize {
        self.lines.get(self.position.row).map_or(0, Line::col_width)
    }
    fn get_lines_count(&self) -> usize {
        self.lines.len()
    }
    #[allow(clippy::arithmetic_side_effects)]
    // allow this because boundary condition is confirmed by myself
    fn move_prev_grapheme(&mut self) {
        self.position.col = self.caret_snap_on_line(&self.lines).col;
        if self.position.col > 0 {
            self.position.col -= 1;
            self.position.col = self.caret_snap_on_line(&self.lines).col;
        } else if self.position.row > 0 {
            self.move_prev_line(1);
            self.position.col = self.get_current_line_col_width();
        }
    }
    fn move_next_grapheme(&mut self) {
        self.position.col = self.caret_snap_on_line(&self.lines).col;

        let step = if let Some(line) = self.lines.get(self.position.row) {
            if let Some(fragment) = line.get_fragment_by_col_idx(self.position.col) {
                fragment.width()
            } else {
                1
            }
        } else {
            0
        };

        if self.position.col < self.get_current_line_col_width() {
            self.position.col = self.position.col.saturating_add(step);
        } else if self.position.row < self.get_lines_count() {
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
            self.get_current_line_col_width(),
        );
    }
    fn move_prev_line(&mut self, step: usize) {
        if self.position.row > 0 {
            self.position.row = self.position.row.saturating_sub(step);
        }
    }
    fn move_next_line(&mut self, step: usize) {
        if self.position.row < self.get_lines_count() {
            self.position.row = min(
                self.position.row.saturating_add(step),
                self.get_lines_count(),
            );
        }
    }
    fn scroll_into_view(&mut self, size: Size) {
        let Position { col, row } = self.caret_snap_on_line(&self.lines);
        let Size { width, height } = size;
        // horizontal
        if col < self.render_offset.col {
            self.render_offset.col = col;
            self.ensure_redraw();
        } else if col >= self.render_offset.col.saturating_add(width) {
            self.render_offset.col = col.saturating_add(1).saturating_sub(width);
            self.ensure_redraw();
        }
        // vertical
        if row < self.render_offset.row {
            self.render_offset.row = row;
            self.ensure_redraw();
        } else if row >= self.render_offset.row.saturating_add(height) {
            self.render_offset.row = row.saturating_add(1).saturating_sub(height);
            self.ensure_redraw();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let result = Buffer::load("tests/fixtures/load.md");
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].content(), "# this is test file for load");
        assert_eq!(lines[1].content(), "");
        assert_eq!(lines[2].content(), "this is sample text");
    }

    #[test]
    fn test_move_prev_grapheme() {
        let mut buffer = Buffer::default();
        let size = Size::new(10, 10);

        buffer.lines = Buffer::gen_lines("a\nb\n");
        buffer.position = Position::new(1, 1);
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 1)); // wrap
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(1, 0));
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 0));
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 0)); // boundary

        // test for full-width character
        buffer.lines = Buffer::gen_lines("aあbい\nうcえd\n");
        buffer.position = Position::new(6, 1);
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(5, 1));
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(3, 1));
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(2, 1));
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 1));
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(6, 0));
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(4, 0));
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(3, 0));
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(1, 0));
        buffer.move_position(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 0));
    }

    #[test]
    fn test_move_next_grapheme() {
        let mut buffer = Buffer::default();
        let size = Size::new(10, 10);
        buffer.lines = Buffer::gen_lines("a\nb\n");
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(1, 0));
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 1));
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(1, 1)); // wrap
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 2));
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 2)); // boundary

        // test for full-width character
        buffer.lines = Buffer::gen_lines("aあbい\nうcえd\n");
        buffer.position = Position::default();
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(1, 0));
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(3, 0));
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(4, 0));
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(6, 0));
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 1));
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(2, 1));
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(3, 1));
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(5, 1));
        buffer.move_position(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(6, 1));
    }

    #[test]
    fn test_move_prev_line() {
        let mut buffer = Buffer::default();
        let size = Size::new(10, 10);
        buffer.lines = Buffer::gen_lines("this\nis\ntest.\n");
        buffer.position = Position::new(3, 2);
        buffer.move_position(size, Up);
        assert_eq!(buffer.caret_screen_position(), Position::new(2, 1)); // snap
        buffer.move_position(size, Up);
        assert_eq!(buffer.caret_screen_position(), Position::new(3, 0));
        buffer.move_position(size, Up);
        assert_eq!(buffer.caret_screen_position(), Position::new(3, 0)); // boundary

        // test for full-width character
        buffer.lines = Buffer::gen_lines("aあ\nいb\ncう\nえe\n");
        buffer.position = Position::new(2, 3);
        buffer.move_position(size, Up);
        assert_eq!(buffer.caret_screen_position(), Position::new(1, 2));
        buffer.move_position(size, Up);
        assert_eq!(buffer.caret_screen_position(), Position::new(2, 1));
        buffer.move_position(size, Up);
        assert_eq!(buffer.caret_screen_position(), Position::new(1, 0));
    }

    #[test]
    fn test_move_next_line() {
        let mut buffer = Buffer::default();
        let size = Size::new(10, 10);
        buffer.lines = Buffer::gen_lines("this\nis\ntest.\n");
        buffer.position = Position::new(3, 0);
        buffer.move_position(size, Down);
        assert_eq!(buffer.caret_screen_position(), Position::new(2, 1)); // snap
        buffer.move_position(size, Down);
        assert_eq!(buffer.caret_screen_position(), Position::new(3, 2));
        buffer.move_position(size, Down);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 3));
        buffer.move_position(size, Down);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 3)); // boundary

        // test for full-width character
        buffer.lines = Buffer::gen_lines("aあ\nいb\ncう\nえe\n");
        buffer.position = Position::new(1, 0);
        buffer.move_position(size, Down);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 1));
        buffer.move_position(size, Down);
        assert_eq!(buffer.caret_screen_position(), Position::new(1, 2));
        buffer.move_position(size, Down);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 3));
    }

    #[test]
    fn test_scroll() {
        let mut buffer = Buffer::default();
        let size = Size::new(2, 2);
        buffer.lines = Buffer::gen_lines("ab\ncd\n");
        buffer.scroll_screen(size, Down);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 0));
        assert_eq!(buffer.render_offset, Position::new(0, 1));
        assert_eq!(buffer.needs_redraw, true);
        buffer.needs_redraw = false;
        buffer.scroll_screen(size, Down);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 1));
        assert_eq!(buffer.render_offset, Position::new(0, 1));
        assert_eq!(buffer.needs_redraw, false);
        buffer.needs_redraw = false;
        buffer.scroll_screen(size, Up);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 1));
        assert_eq!(buffer.render_offset, Position::new(0, 0));
        assert_eq!(buffer.needs_redraw, true);
        buffer.needs_redraw = false;
        buffer.scroll_screen(size, Up);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 0));
        assert_eq!(buffer.render_offset, Position::new(0, 0));
        assert_eq!(buffer.needs_redraw, false);
        buffer.needs_redraw = false;

        buffer.scroll_screen(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 0));
        assert_eq!(buffer.render_offset, Position::new(1, 0));
        assert_eq!(buffer.needs_redraw, true);
        buffer.needs_redraw = false;
        buffer.scroll_screen(size, Right);
        assert_eq!(buffer.caret_screen_position(), Position::new(1, 0));
        assert_eq!(buffer.render_offset, Position::new(1, 0));
        assert_eq!(buffer.needs_redraw, false);
        buffer.needs_redraw = false;
        buffer.scroll_screen(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(1, 0));
        assert_eq!(buffer.render_offset, Position::new(0, 0));
        assert_eq!(buffer.needs_redraw, true);
        buffer.needs_redraw = false;
        buffer.scroll_screen(size, Left);
        assert_eq!(buffer.caret_screen_position(), Position::new(0, 0));
        assert_eq!(buffer.render_offset, Position::new(0, 0));
        assert_eq!(buffer.needs_redraw, false);
        buffer.needs_redraw = false;
    }
}
