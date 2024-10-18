use super::file_info::FileType;
use super::terminal::Terminal;
use crate::editor::{Editor, Mode};
use std::io::Error;
use unicode_width::UnicodeWidthStr;

#[derive(Default, Eq, PartialEq)]
pub struct DocumentStatus {
    file_name: Option<String>,
    file_type: Option<FileType>,
    total_lines: usize,
    total_cols: usize,
    current_line_idx: usize,
    current_col_idx: usize,
    mode: Mode,
}

impl DocumentStatus {
    pub fn from(editor: &Editor) -> Self {
        let current_line_idx = editor.current_view().cursor.line_idx();
        DocumentStatus {
            file_name: editor.current_view().buffer.file_info.get_file_name(),
            file_type: editor.current_view().buffer.file_info.get_file_type(),
            total_lines: editor.current_view().buffer.get_lines_count(),
            total_cols: editor
                .current_view()
                .buffer
                .get_line_col_width(current_line_idx),
            current_line_idx,
            current_col_idx: editor.current_view().cursor.col_idx(),
            mode: editor.mode,
        }
    }
    pub fn file_name_string(&self) -> String {
        self.file_name
            .clone()
            .unwrap_or_else(|| String::from("No Name"))
    }
    pub fn file_type_string(&self) -> String {
        self.file_type.as_ref().map_or_else(
            || String::from("No Type"),
            |file_type| format!("{file_type:?}"),
        )
    }
    pub fn lines_info_string(&self) -> String {
        format!("{}/{}", self.current_line_idx, self.total_lines)
    }
    pub fn cols_info_string(&self) -> String {
        format!("{}/{}", self.current_col_idx, self.total_cols)
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
            " {:?} | {}",
            self.document_status.mode,
            self.document_status.file_name_string()
        );
        let right = format!(
            "{} | {}|{} ",
            self.document_status.file_type_string(),
            self.document_status.lines_info_string(),
            self.document_status.cols_info_string()
        );

        let width = Terminal::size().unwrap_or_default().width as usize;
        // minus 1 for the space between left and right
        let reminder_len = width.saturating_sub(left.width()).saturating_sub(1);
        let line_text = format!("{left} {right:>reminder_len$}");
        Terminal::print_invert_row(line_idx, &line_text)?;
        self.needs_redraw = false;
        Ok(())
    }
}
