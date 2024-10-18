use super::terminal::Terminal;
use crate::editor::{Editor, Mode, NAME, VERSION};
use std::io::Error;

#[derive(Default, Eq, PartialEq)]
pub struct DocumentStatus {
    file_name: Option<String>,
    file_type: Option<String>,
    total_lines: usize,
    // current_col_idx: usize,
    current_line_idx: usize,
    mode: Mode,
}

impl DocumentStatus {
    pub fn from(editor: &Editor) -> Self {
        DocumentStatus {
            file_name: editor.current_view().buffer.file_info.get_file_name(),
            file_type: editor.current_view().buffer.file_info.get_file_type(),
            total_lines: editor.current_view().buffer.get_lines_count(),
            // current_col_idx: editor.current_view().cursor.col_idx(),
            current_line_idx: editor.current_view().cursor.line_idx(),
            mode: editor.mode,
        }
    }
    pub fn file_name_string(&self) -> String {
        self.file_name
            .clone()
            .unwrap_or_else(|| String::from("[No Name]"))
    }
    pub fn total_lines_string(&self) -> String {
        format!("{} lines", self.total_lines)
    }
    pub fn position_string(&self) -> String {
        format!("{}/{}", self.current_line_idx, self.total_lines)
    }
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
    pub fn update_status(&mut self, status: DocumentStatus) {
        if self.document_status != status {
            self.document_status = status;
            self.needs_redraw = true;
        }
    }
    pub fn render(&mut self, line_idx: usize) -> Result<(), Error> {
        if !self.needs_redraw {
            return Ok(());
        }

        let left = format!(
            "{:?} | {}",
            self.document_status.mode,
            self.document_status.file_name_string()
        );
        let right = format!(
            "{NAME} - {VERSION} | {} {}",
            self.document_status.position_string(),
            self.document_status.total_lines_string()
        );
        let line_text = format!("{left} | {right}");
        Terminal::print_row(line_idx, &line_text)?;
        self.needs_redraw = false;
        Ok(())
    }
}
