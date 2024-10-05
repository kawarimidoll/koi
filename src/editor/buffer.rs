use super::line::Line;
use super::terminal::{Position, Size};
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
        let top = offset.row;
        let left = offset.col;
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
    pub fn get_line_col_width(&self, row: usize) -> usize {
        self.lines.get(row).map_or(0, Line::col_width)
    }
    pub fn get_lines_count(&self) -> usize {
        self.lines.len()
    }

    pub fn insert(&mut self, str: &str, at: Position) -> bool {
        let Position { col, row } = at;

        // out of bounds
        if row > self.get_lines_count() {
            return false;
        }

        // append a new line
        if row == self.get_lines_count() {
            self.lines.push(Line::from(str));
            return true;
        }

        // insert a new character in an existing line
        if let Some(line) = self.lines.get_mut(row) {
            line.insert(col, str);
            return true;
        }

        // maybe dead code, but the compiler doesn't know that
        false
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
        buffer.insert("ok", Position { col: 1, row: 0 });
        assert_eq!(buffer.lines[0].content(), "tokhis");
    }
}
