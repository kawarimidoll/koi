use super::file_info::FileInfo;
use super::line::Line;
use super::position::Position;
use super::size::Size;
use std::cmp::min;
use std::fs::{read_to_string, File};
use std::io::{Error, Write};

pub struct Buffer {
    pub lines: Vec<Line>,
    pub needs_redraw: bool,
    pub file_info: FileInfo,
    pub modified_count: usize,
}

impl Buffer {
    pub fn from_file(path: &str) -> Self {
        Self {
            lines: Self::load(path).unwrap_or_default(),
            file_info: FileInfo::from(path),
            ..Self::default()
        }
    }
    #[cfg(test)]
    pub fn from_string(str: &str) -> Self {
        Buffer {
            lines: Self::gen_lines(str),
            ..Self::default()
        }
    }
    pub fn load(path: &str) -> Result<Vec<Line>, Error> {
        Ok(Self::gen_lines(&read_to_string(path)?))
    }
    pub fn gen_lines(src: &str) -> Vec<Line> {
        src.lines().map(Line::from).collect()
    }
    pub fn ensure_redraw(&mut self) {
        self.needs_redraw = true;
    }
    pub fn has_path(&self) -> bool {
        self.file_info.has_path()
    }
    pub fn save_as(&mut self, path: &str) -> Result<(), Error> {
        self.file_info = FileInfo::from(path);
        self.save()
    }
    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(path) = &self.file_info.get_path() {
            let mut file = File::create(path)?;
            for line in &self.lines {
                writeln!(file, "{}", line.content())?;
            }
            self.modified_count = 0;
            Ok(())
        } else {
            Err(Error::new(std::io::ErrorKind::Other, "No file path"))
        }
    }
    pub fn render<F: Fn(usize, &str) -> Result<(), Error>>(
        &mut self,
        size: Size,
        offset: Position,
        renderer: F,
    ) -> Result<(), Error> {
        // render function
        if !self.needs_redraw || size.width == 0 || size.height == 0 {
            return Ok(());
        }
        let top = offset.line_idx;
        let left = offset.col_idx;
        let right = left.saturating_add(size.width);
        for current_row in 0..size.height {
            let current_line = top.saturating_add(current_row);
            if let Some(line) = self.lines.get(current_line) {
                let end = min(right, line.col_width());
                let str = line.get_str_by_col_range(left..end);
                renderer(current_row, &str)?;
                continue;
            }
            renderer(current_row, "~")?;
        }
        self.needs_redraw = false;
        Ok(())
    }
    pub fn get_line_col_width(&self, line_idx: usize) -> usize {
        self.lines.get(line_idx).map_or(0, Line::col_width)
    }
    pub fn get_lines_count(&self) -> usize {
        self.lines.len()
    }

    fn increase_modified_count(&mut self) {
        self.modified_count = self.modified_count.saturating_add(1);
    }

    // TODO add test
    pub fn set_line(&mut self, str: &str, line_idx: usize) {
        let line = Line::from(str);
        if line_idx >= self.get_lines_count() {
            self.lines.push(line);
        } else {
            self.lines[line_idx] = line;
        }
        self.increase_modified_count();
        self.ensure_redraw();
    }
    pub fn remove_line(&mut self, line_idx: usize) {
        self.lines.remove(line_idx);
        self.increase_modified_count();
        self.ensure_redraw();
    }
    pub fn cutoff_line(&mut self, at: Position) {
        let Position { line_idx, col_idx } = at;
        if line_idx >= self.get_lines_count() {
            return;
        }
        // we have a valid line_idx
        self.lines[line_idx].split_off(col_idx);
        self.increase_modified_count();
        self.ensure_redraw();
    }
    pub fn insert(&mut self, str: &str, at: Position) -> bool {
        let Position { line_idx, col_idx } = at;

        // out of bounds
        if line_idx > self.get_lines_count() {
            return false;
        }

        if line_idx == self.get_lines_count() {
            // append a new line
            self.lines.push(Line::from(str));
        } else if let Some(line) = self.lines.get_mut(line_idx) {
            // insert a new character in an existing line
            line.insert(col_idx, str);
        } else {
            // maybe dead code, but the compiler doesn't know that
            return false;
        }
        self.increase_modified_count();
        self.ensure_redraw();
        true
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
        self.increase_modified_count();
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
        self.increase_modified_count();
        true
    }
}
impl Default for Buffer {
    fn default() -> Self {
        Buffer {
            lines: Vec::new(),
            needs_redraw: true,
            file_info: FileInfo::default(),
            modified_count: 0,
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
