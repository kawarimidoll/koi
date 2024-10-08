use super::line::Line;
use super::position::Position;
use super::size::Size;
use std::{cmp::min, fs::read_to_string, io::Error};

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    pub needs_redraw: bool,
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
    pub fn gen_lines(src: &str) -> Vec<Line> {
        src.lines().map(Line::from).collect()
    }
    pub fn ensure_redraw(&mut self) {
        self.needs_redraw = true;
    }
    pub fn render<F: Fn(usize, &str) -> Result<(), Error>>(
        &mut self,
        size: Size,
        offset: Position,
        renderer: F,
    ) -> Result<(), Error> {
        // render function
        if !self.needs_redraw {
            return Ok(());
        }
        let Size { width, height } = size;
        let top = offset.line_idx;
        let left = offset.col_idx;
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
    pub fn get_line_col_width(&self, line_idx: usize) -> usize {
        self.lines.get(line_idx).map_or(0, Line::col_width)
    }
    pub fn get_lines_count(&self) -> usize {
        self.lines.len()
    }

    pub fn insert(&mut self, str: &str, at: Position) -> bool {
        let Position { line_idx, col_idx } = at;

        // out of bounds
        if line_idx > self.get_lines_count() {
            return false;
        }

        // append a new line
        if line_idx == self.get_lines_count() {
            self.lines.push(Line::from(str));
            return true;
        }

        // insert a new character in an existing line
        if let Some(line) = self.lines.get_mut(line_idx) {
            line.insert(col_idx, str);
            return true;
        }

        // maybe dead code, but the compiler doesn't know that
        false
    }
    pub fn insert_newline(&mut self, at: Position) -> bool {
        let Position { line_idx, col_idx } = at;
        if line_idx >= self.get_lines_count() {
            self.lines.push(Line::default());
        } else {
            // we have a valid line_idx
            let second_half = self.lines[line_idx].split_off(col_idx);
            self.lines.insert(line_idx.saturating_add(1), second_half);
        }
        true
    }
    pub fn remove_char(&mut self, at: Position) -> bool {
        let Position { line_idx, col_idx } = at;
        // out of bounds
        if line_idx >= self.get_lines_count() {
            return false;
        }

        // below here, we have a valid line_idx
        if col_idx < self.lines[line_idx].col_width() {
            // remove a character
            self.lines[line_idx].remove(col_idx, 1);
        } else if line_idx < self.get_lines_count().saturating_sub(1) {
            // remove a newline (merge two lines)
            let next_line = self.lines.remove(line_idx.saturating_add(1));
            self.lines[line_idx].append(&next_line);
        } else {
            // the last line, the last character
            return false;
        }
        true
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
    fn test_insert() {
        let mut buffer = Buffer::default();
        buffer.lines = Buffer::gen_lines("this\nis\ntest.\n");
        buffer.insert("ok", Position::new(0, 1));
        assert_eq!(buffer.lines[0].content(), "tokhis");

        let mut buffer = Buffer::default();
        buffer.lines = Buffer::gen_lines("qwert");
        buffer.insert("\t", Position::new(0, 1));
        assert_eq!(buffer.lines[0].content(), "q\twert");
        buffer.insert("a", Position::new(0, 4));
        assert_eq!(buffer.lines[0].content(), "q\tawert");
    }

    #[test]
    fn test_insert_newline() {
        let mut buffer = Buffer::default();
        buffer.lines = Buffer::gen_lines("this\nis\ntest.\n");
        buffer.insert_newline(Position::new(1, 0));
        assert_eq!(buffer.lines.len(), 4);
        assert_eq!(buffer.lines[1].content(), "");
        assert_eq!(buffer.lines[2].content(), "is");
        buffer.insert_newline(Position::new(2, 2));
        assert_eq!(buffer.lines.len(), 5);
        assert_eq!(buffer.lines[2].content(), "is");
        assert_eq!(buffer.lines[3].content(), "");
        buffer.insert_newline(Position::new(2, 1));
        assert_eq!(buffer.lines.len(), 6);
        assert_eq!(buffer.lines[2].content(), "i");
        assert_eq!(buffer.lines[3].content(), "s");
    }

    #[test]
    fn test_remove_char() {
        let mut buffer = Buffer::default();
        buffer.lines = Buffer::gen_lines("this\nis\ntest.\n");
        buffer.remove_char(Position::new(1, 0));
        assert_eq!(buffer.lines.len(), 3);
        assert_eq!(buffer.lines[1].content(), "s");
        buffer.remove_char(Position::new(1, 1));
        assert_eq!(buffer.lines.len(), 2);
        assert_eq!(buffer.lines[1].content(), "stest.");
    }
}
