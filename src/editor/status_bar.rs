use super::terminal::Terminal;
use crate::editor::{Mode, NAME, VERSION};
use std::io::Error;

#[derive(Default, Eq, PartialEq)]
pub struct DocumentStatus {
    mode: Mode,
}

#[derive(Default)]
pub struct StatusBar {
    document_status: DocumentStatus,
    needs_redraw: bool,
}
impl StatusBar {
    pub fn new() -> Self {
        Self {
            needs_redraw: true,
            ..Self::default()
        }
    }
    pub fn update_status(&mut self, mode: Mode) {
        let new_status = DocumentStatus { mode };
        if self.document_status != new_status {
            self.document_status = new_status;
            self.needs_redraw = true;
        }
    }
    pub fn render(&mut self, line_idx: usize) -> Result<(), Error> {
        if !self.needs_redraw {
            return Ok(());
        }

        let left = format!("{:?}", self.document_status.mode);
        let right = format!("{NAME} - {VERSION}");
        let line_text = format!("{left} {right}");
        Terminal::print_row(line_idx, &line_text)?;
        self.needs_redraw = false;
        Ok(())
    }
}
