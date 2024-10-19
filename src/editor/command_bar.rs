use super::terminal::Terminal;
use super::Mode;
use std::io::Error;
use unicode_width::UnicodeWidthStr;

pub struct CommandBar {
    prompt: String,
    value: String,
    needs_redraw: bool,
    pub caret_col: usize,
    pub mode: Mode,
}
impl CommandBar {
    pub fn new(mode: Mode) -> Self {
        debug_assert!(mode == Mode::Command || mode == Mode::Search);
        let prompt = if mode == Mode::Command { ":" } else { "/" };
        Self {
            prompt: prompt.to_string(),
            value: String::default(),
            needs_redraw: true,
            caret_col: prompt.width(),
            mode,
        }
    }
    pub fn insert(&mut self, c: char) {
        self.value.push(c);
        self.needs_redraw = true;
        self.caret_col = self.text().width();
    }
    pub fn delete_backward(&mut self) {
        self.value.pop();
        self.needs_redraw = true;
        self.caret_col = self.text().width();
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn text(&self) -> String {
        format!("{}{}", &self.prompt, &self.value)
    }
    pub fn render(&mut self, bottom_line: usize) -> Result<(), Error> {
        if !self.needs_redraw {
            return Ok(());
        }
        Terminal::print_row(bottom_line, &self.text())?;
        self.needs_redraw = false;
        Ok(())
    }
}
