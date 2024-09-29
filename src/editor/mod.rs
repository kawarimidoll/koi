// 日本語🇯🇵の表示テスト
use crossterm::event::{
    read,
    Event::{Key, Resize},
    KeyCode::{self, Char, Down, End, Home, Left, PageDown, PageUp, Right, Up},
    KeyEvent, KeyModifiers,
};
use std::{cmp::min, fs::read_to_string, io::Error};
use terminal::{Position, Size, Terminal};
mod terminal;
use line::Line;
mod line;
mod text_fragment;

#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    // position is col-row vertex from the document origin
    position: Position,
    render_offset: Position,
    lines: Vec<Line>,
    needs_redraw: bool,
    size: Size,
}

impl Editor {
    pub fn new() -> Result<Self, Error> {
        let current_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Terminal::terminate();
            current_hook(panic_info);
        }));
        Terminal::initialize()?;
        let mut editor = Self::default();

        editor.handle_args();
        editor.needs_redraw = true;
        editor.size = Terminal::size().unwrap_or_default();

        Ok(editor)
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
        src.lines().map(|str| Line::from(str)).collect()
    }

    pub fn run(&mut self) {
        self.repl();
    }

    fn repl(&mut self) {
        let bottom_line = self.size.height.saturating_sub(1);
        Terminal::print_row(bottom_line, "Type something. Press 'q' to quit.").unwrap();
        self.move_caret();

        loop {
            self.refresh_screen();
            if self.should_quit {
                break;
            }
            match read() {
                Ok(Key(key_event)) => {
                    // necessary for windows
                    if key_event.kind == crossterm::event::KeyEventKind::Press {
                        self.handle_key_event(key_event);
                    }
                }
                Ok(Resize(width16, height16)) => {
                    self.handle_resize_event(width16, height16);
                }
                Err(err) => {
                    Terminal::print_row(bottom_line, &format!("{err}")).unwrap();
                }
                _ => {
                    Terminal::print_row(bottom_line, "Unsupported event!").unwrap();
                }
            }
        }
    }

    fn handle_key_event(&mut self, event: KeyEvent) {
        let height = self.size.height;
        let KeyEvent {
            code, modifiers, ..
        } = event;

        match code {
            Char('q') if modifiers == KeyModifiers::NONE => self.should_quit = true,

            Left | Down | Right | Up | Home | End | PageDown | PageUp => {
                self.move_position(code);
            }
            _ => (),
        }
        let Position {
            col: doc_x,
            row: doc_y,
        } = self.position;
        let Position { col, row } = self.caret_screen_position();
        let Position {
            col: off_c,
            row: off_r,
        } = self.render_offset;

        let info = if let Some(line) = self.lines.get(self.position.row) {
            if let Some(fragment) = line.get_fragment_by_col_idx(self.position.col) {
                &format!(
                    "{}, {}, {}",
                    fragment.grapheme,
                    fragment.width(),
                    fragment.left_col_width
                )
            } else {
                ""
            }
        } else {
            ""
        };

        let _ = Terminal::print_row(
            height - 1,
            &format!("loc: {doc_x},{doc_y}, pos: {col},{row}, off: {off_c},{off_r}, [{info}]"),
        );
    }
    #[allow(clippy::as_conversions)]
    fn handle_resize_event(&mut self, width16: u16, height16: u16) {
        let width = width16 as usize;
        let height = height16 as usize;
        // let _ = Terminal::print_row(height - 1, &format!("Resize to: {width:?}, {height:?}"));
        self.size = Size { width, height };
        self.needs_redraw = true;
    }
    fn refresh_screen(&mut self) {
        let _ = Terminal::hide_caret();
        let _ = self.render(self.size, Terminal::print_row);
        self.move_caret();
        let _ = Terminal::show_caret();
        let _ = Terminal::execute();
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
    fn move_caret(&self) {
        Terminal::move_caret_to(self.caret_screen_position()).unwrap();
    }
    fn caret_screen_position(&self) -> Position {
        self.caret_snap_on_line()
            .saturating_sub(&self.render_offset)
    }
    fn caret_snap_on_line(&self) -> Position {
        let col = if let Some(line) = self.lines.get(self.position.row) {
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
    fn move_position(&mut self, code: KeyCode) {
        match code {
            Left => self.move_left(),
            Right => self.move_right(),
            Up => self.move_up(1),
            Down => self.move_down(1),
            Home => self.position.col = 0,
            End => self.position.col = usize::MAX,
            PageUp => {
                let off_r = self.render_offset.row;
                self.move_up(self.size.height);
                self.render_offset.row = off_r.saturating_sub(self.size.height);
                self.needs_redraw = true;
            }
            PageDown => {
                let off_r = self.render_offset.row;
                self.move_down(self.size.height);
                self.render_offset.row = min(
                    off_r.saturating_add(self.size.height),
                    self.get_lines_count().saturating_sub(self.size.height),
                );
                self.needs_redraw = true;
            }
            _ => (),
        };
        self.scroll_into_view();
    }
    fn get_current_line_col_width(&self) -> usize {
        self.lines.get(self.position.row).map_or(0, Line::col_width)
    }
    fn get_lines_count(&self) -> usize {
        self.lines.len()
    }
    #[allow(clippy::arithmetic_side_effects)]
    // allow this because check boundary condition by myself
    fn move_left(&mut self) {
        self.position.col = self.caret_snap_on_line().col;
        if self.position.col > 0 {
            self.position.col -= 1;
            self.position.col = self.caret_snap_on_line().col;
        } else if self.position.row > 0 {
            self.move_up(1);
            self.position.col = self.get_current_line_col_width();
        }
    }
    fn move_right(&mut self) {
        self.position.col = self.caret_snap_on_line().col;

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
            self.move_down(1);
            self.position.col = 0;
        }
    }
    fn move_up(&mut self, step: usize) {
        if self.position.row > 0 {
            self.position.row = self.position.row.saturating_sub(step);
        }
    }
    fn move_down(&mut self, step: usize) {
        if self.position.row < self.get_lines_count() {
            self.position.row = min(
                self.position.row.saturating_add(step),
                self.get_lines_count(),
            );
        }
    }
    fn scroll_into_view(&mut self) {
        let Position { col, row } = self.caret_snap_on_line();
        let Size { width, height } = self.size;
        // horizontal
        if col < self.render_offset.col {
            self.render_offset.col = col;
            self.needs_redraw = true;
        } else if col >= self.render_offset.col.saturating_add(width) {
            self.render_offset.col = col.saturating_add(1).saturating_sub(width);
            self.needs_redraw = true;
        }
        // vertical
        if row < self.render_offset.row {
            self.render_offset.row = row;
            self.needs_redraw = true;
        } else if row >= self.render_offset.row.saturating_add(height) {
            self.render_offset.row = row.saturating_add(1).saturating_sub(height);
            self.needs_redraw = true;
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        let _ = Terminal::terminate();
        if self.should_quit {
            let _ = Terminal::print_row(0, "Goodbye, koi!\r\n");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let result = Editor::load("tests/fixtures/load.md");
        assert!(result.is_ok());
        let lines = result.unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].content(), "# this is test file for load");
        assert_eq!(lines[1].content(), "");
        assert_eq!(lines[2].content(), "this is sample text");
    }

    #[test]
    fn test_resize() {
        let mut editor = Editor::default();
        assert_eq!(editor.size, Size::default());
        editor.handle_resize_event(10, 10);
        assert_eq!(editor.size, Size::new(10, 10));
    }

    #[test]
    fn test_move_left() {
        let mut editor = Editor::default();
        editor.size = Size::new(10, 10);

        editor.lines = Editor::gen_lines("a\nb\n");
        editor.position = Position::new(1, 1);
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 1)); // wrap
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(1, 0));
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 0));
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 0)); // boundary

        // test for full-width character
        editor.lines = Editor::gen_lines("aあbい\nうcえd\n");
        editor.position = Position::new(6, 1);
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(5, 1));
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(3, 1));
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(2, 1));
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 1));
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(6, 0));
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(4, 0));
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(3, 0));
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(1, 0));
        editor.move_position(Left);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 0));
    }

    #[test]
    fn test_move_right() {
        let mut editor = Editor::default();
        editor.size = Size::new(10, 10);
        editor.lines = Editor::gen_lines("a\nb\n");
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(1, 0));
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 1));
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(1, 1)); // wrap
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 2));
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 2)); // boundary

        // test for full-width character
        editor.lines = Editor::gen_lines("aあbい\nうcえd\n");
        editor.position = Position::default();
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(1, 0));
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(3, 0));
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(4, 0));
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(6, 0));
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 1));
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(2, 1));
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(3, 1));
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(5, 1));
        editor.move_position(Right);
        assert_eq!(editor.caret_screen_position(), Position::new(6, 1));
    }

    #[test]
    fn test_move_up() {
        let mut editor = Editor::default();
        editor.size = Size::new(10, 10);
        editor.lines = Editor::gen_lines("this\nis\ntest.\n");
        editor.position = Position::new(3, 2);
        editor.move_position(Up);
        assert_eq!(editor.caret_screen_position(), Position::new(2, 1)); // snap
        editor.move_position(Up);
        assert_eq!(editor.caret_screen_position(), Position::new(3, 0));
        editor.move_position(Up);
        assert_eq!(editor.caret_screen_position(), Position::new(3, 0)); // boundary

        // test for full-width character
        editor.lines = Editor::gen_lines("aあ\nいb\ncう\nえe\n");
        editor.position = Position::new(2, 3);
        editor.move_position(Up);
        assert_eq!(editor.caret_screen_position(), Position::new(1, 2));
        editor.move_position(Up);
        assert_eq!(editor.caret_screen_position(), Position::new(2, 1));
        editor.move_position(Up);
        assert_eq!(editor.caret_screen_position(), Position::new(1, 0));
    }

    #[test]
    fn test_move_down() {
        let mut editor = Editor::default();
        editor.size = Size::new(10, 10);
        editor.lines = Editor::gen_lines("this\nis\ntest.\n");
        editor.position = Position::new(3, 0);
        editor.move_position(Down);
        assert_eq!(editor.caret_screen_position(), Position::new(2, 1)); // snap
        editor.move_position(Down);
        assert_eq!(editor.caret_screen_position(), Position::new(3, 2));
        editor.move_position(Down);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 3));
        editor.move_position(Down);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 3)); // boundary

        // test for full-width character
        editor.lines = Editor::gen_lines("aあ\nいb\ncう\nえe\n");
        editor.position = Position::new(1, 0);
        editor.move_position(Down);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 1));
        editor.move_position(Down);
        assert_eq!(editor.caret_screen_position(), Position::new(1, 2));
        editor.move_position(Down);
        assert_eq!(editor.caret_screen_position(), Position::new(0, 3));
    }
}
