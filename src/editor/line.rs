use super::text_fragment::TextFragment;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

// https://rust-lang.github.io/rust-clippy/master/index.html#/format_collect
use std::fmt::Write;

const ELLIPSIS_LEFT: &str = "«";
const ELLIPSIS_RIGHT: &str = "»";

#[derive(Clone, Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
    string: String,
    col_width: usize,
}

#[allow(dead_code)]
impl Line {
    pub fn from(string: &str) -> Self {
        debug_assert!(string.is_empty() || string.lines().count() == 1);
        let mut line = Self::default();
        line.string = String::from(string);
        line.rebuild_fragments();
        line
    }
    fn rebuild_fragments(&mut self) {
        let mut left_col_width = 0;
        self.fragments = self
            .string
            .graphemes(true)
            .map(|grapheme| {
                let fragment = TextFragment::new(grapheme, left_col_width);
                left_col_width = left_col_width.saturating_add(fragment.width());
                fragment
            })
            .collect();

        self.col_width = left_col_width;
    }
    pub fn content(&self) -> &str {
        &self.string
    }
    pub fn grapheme_count(&self) -> usize {
        self.fragments.len()
    }
    pub fn get_str(&self) -> String {
        self.get_str_by_col_range(0..self.col_width)
    }
    pub fn get_str_by_col_range(&self, range: Range<usize>) -> String {
        if range.start == range.end {
            return String::default();
        }
        // Range<usize> must have start and end
        let mut acc = 0;
        let mut start = 0;
        let mut end = 0;
        let mut set_start = false;
        let mut ellipsis_start = false;
        let mut ellipsis_end = false;
        // println!("range: {:?}", range);
        for (i, fragment) in self.fragments.iter().enumerate() {
            if !set_start && acc >= range.start {
                ellipsis_start = acc > range.start;
                start = i;
                set_start = true;
            }
            acc = acc.saturating_add(fragment.width());
            if set_start && acc >= range.end {
                if acc > range.end {
                    ellipsis_end = true;
                    end = i;
                } else {
                    end = i.saturating_add(1);
                }
                break;
            }
        }
        // println!("start: {start}, end: {end}, acc: {acc}");
        format!(
            "{}{}{}",
            if ellipsis_start { ELLIPSIS_LEFT } else { "" },
            self.fragments[start..end]
                .iter()
                .fold(String::new(), |mut output, fragment| {
                    let _ = write!(output, "{fragment}");
                    output
                }),
            if ellipsis_end { ELLIPSIS_RIGHT } else { "" },
        )
    }
    pub fn get_fragment_by_col_idx(&self, col_idx: usize) -> Option<&TextFragment> {
        let mut acc: usize = 0;
        for fragment in &self.fragments {
            acc = acc.saturating_add(fragment.width());
            if acc > col_idx {
                return Some(fragment);
            }
        }
        None
    }
    pub fn grapheme_idx_to_col_idx(&self, grapheme_idx: usize) -> usize {
        self.fragments
            .iter()
            .take(grapheme_idx)
            .map(TextFragment::width)
            .sum()
    }
    pub fn col_idx_to_grapheme_idx(&self, col_idx: usize) -> usize {
        let mut acc: usize = 0;
        let mut grapheme_idx: usize = 0;
        for fragment in &self.fragments {
            if acc >= col_idx {
                break;
            }
            acc = acc.saturating_add(fragment.width());
            grapheme_idx = grapheme_idx.saturating_add(1);
        }
        return grapheme_idx;
    }
    pub fn col_width(&self) -> usize {
        self.col_width
    }
    // TODO: needs performance improvement... obviously not efficient
    pub fn split_off(&mut self, at_col_idx: usize) -> Self {
        if at_col_idx == 0 {
            let remainder = self.clone();
            self.string.clear();
            self.fragments.clear();
            self.col_width = 0;
            return remainder;
        }
        let mut acc: usize = 0;
        let mut byte_len: usize = 0;
        for fragment in &self.fragments {
            acc = acc.saturating_add(fragment.width());
            byte_len = byte_len.saturating_add(fragment.grapheme().len());
            if acc >= at_col_idx {
                break;
            }
        }
        let remainder = self.string.split_off(byte_len);
        self.rebuild_fragments();
        Self::from(&remainder)
    }
    pub fn insert(&mut self, at_col_idx: usize, string: &str) {
        if at_col_idx < self.col_width {
            // we can't use get_str_by_col_range here because we need special handling for tabs
            let substr = if at_col_idx == 0 {
                String::default()
            } else {
                let mut acc: usize = 0;
                let mut end = 0;
                for (i, fragment) in self.fragments.iter().enumerate() {
                    acc = acc.saturating_add(fragment.width());
                    if acc >= at_col_idx {
                        end = i.saturating_add(1);
                        break;
                    }
                }
                self.fragments[0..end]
                    .iter()
                    .fold(String::new(), |mut output, fragment| {
                        if fragment.grapheme() == "\t" {
                            let _ = write!(output, "\t");
                        } else {
                            let _ = write!(output, "{fragment}");
                        }
                        output
                    })
                    .to_string()
            };
            self.string.insert_str(substr.len(), string);
        } else {
            self.string.push_str(string);
        }

        self.rebuild_fragments();
    }
    pub fn append(&mut self, other: &Self) {
        self.insert(self.col_width(), &other.string);
    }
    pub fn remove(&mut self, start_grapheme_idx: usize, grapheme_count: usize) {
        if start_grapheme_idx < self.grapheme_count() {
            let start_byte_idx: usize = self.fragments[0..start_grapheme_idx]
                .iter()
                .map(TextFragment::byte_len)
                .sum();

            let end_grapheme_idx = start_grapheme_idx.saturating_add(grapheme_count);
            if end_grapheme_idx < self.grapheme_count() {
                let end_byte_idx = self.fragments[0..end_grapheme_idx]
                    .iter()
                    .map(TextFragment::byte_len)
                    .sum();
                self.string.drain(start_byte_idx..end_byte_idx);
            } else {
                self.string.drain(start_byte_idx..);
            }
            // TODO: needs performance improvement... obviously not efficient
            self.rebuild_fragments();
            // let diff_width = self
            //     .fragments
            //     .drain(start_grapheme_idx..end_grapheme_idx)
            //     .collect::<Vec<_>>()
            //     .iter()
            //     .map(TextFragment::width)
            //     .sum();
            // self.col_width = self.col_width.saturating_sub(diff_width);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line() {
        let mut line = Line::from("test_from");
        assert_eq!(line.content(), "test_from");
        assert_eq!(line.grapheme_count(), 9);
        assert_eq!(line.col_width(), 9);
        assert_eq!(line.grapheme_idx_to_col_idx(5), 5);
        assert_eq!(
            line.get_fragment_by_col_idx(4)
                .map_or("", |f| &f.grapheme()),
            "_"
        );
        assert_eq!(
            line.get_fragment_by_col_idx(5)
                .map_or("", |f| &f.grapheme()),
            "f"
        );
        assert_eq!(line.get_str_by_col_range(0..0), "");
        assert_eq!(line.get_str_by_col_range(2..6), "st_f");
        assert_eq!(line.get_str_by_col_range(1..5), "est_");
        line.insert(1, "ok");
        assert_eq!(line.content(), "tokest_from");
        assert_eq!(line.grapheme_count(), 11);
        assert_eq!(line.col_width(), 11);
        line.remove(1, 2);
        assert_eq!(line.content(), "test_from");
        assert_eq!(line.grapheme_count(), 9);
        assert_eq!(line.col_width(), 9);

        let mut line = Line::from("こんにちは");
        assert_eq!(line.content(), "こんにちは");
        assert_eq!(line.grapheme_count(), 5);
        assert_eq!(line.col_width(), 10);
        assert_eq!(line.grapheme_idx_to_col_idx(2), 4);
        assert_eq!(
            line.get_fragment_by_col_idx(4)
                .map_or("", |f| &f.grapheme()),
            "に"
        );
        assert_eq!(
            line.get_fragment_by_col_idx(5)
                .map_or("", |f| &f.grapheme()),
            "に"
        );
        assert_eq!(line.get_str_by_col_range(2..6), "んに");
        assert_eq!(line.get_str_by_col_range(1..5), "«ん»");
        line.insert(2, "ok");
        assert_eq!(line.content(), "こokんにちは");
        assert_eq!(line.grapheme_count(), 7);
        assert_eq!(line.col_width(), 12);
        line.remove(2, 2);
        assert_eq!(line.content(), "こoにちは");
        assert_eq!(line.grapheme_count(), 5);
        assert_eq!(line.col_width(), 9);

        let line2 = Line::from("test_from");
        line.append(&line2);
        assert_eq!(line.content(), "こoにちはtest_from");
    }

    #[test]
    fn test_tab() {
        let mut line = Line::from("\t");
        assert_eq!(line.content(), "\t");
        assert_eq!(line.grapheme_count(), 1);
        assert_eq!(line.col_width(), 4);
        assert_eq!(line.get_str(), "→   ");
        line.insert(0, "ok");
        assert_eq!(line.content(), "ok\t");
        assert_eq!(line.grapheme_count(), 3);
        assert_eq!(line.col_width(), 4);
        assert_eq!(line.get_str(), "ok→ ");

        let mut line = Line::from("test_from");
        line.insert(1, "\t");
        assert_eq!(line.content(), "t\test_from");
        assert_eq!(line.col_width(), 12);
        assert_eq!(line.get_str(), "t→  est_from");
        line.insert(4, "\t");
        assert_eq!(line.get_str(), "t→  →   est_from");
        assert_eq!(line.content(), "t\t\test_from");
        line.insert(14, "\t");
        assert_eq!(line.get_str(), "t→  →   est_fr→ om");
        assert_eq!(line.content(), "t\t\test_fr\tom");

        let mut line = Line::from("qwert");
        line.insert(1, "\t");
        assert_eq!(line.content(), "q\twert");
        assert_eq!(line.get_str(), "q→  wert");
        line.insert(4, "a");
        assert_eq!(line.content(), "q\tawert");
        assert_eq!(line.get_str(), "q→  awert");
    }

    #[test]
    fn test_idx_conversion() {
        let line = Line::from("qwert");
        assert_eq!(line.col_idx_to_grapheme_idx(0), 0);
        assert_eq!(line.col_idx_to_grapheme_idx(3), 3);
        assert_eq!(line.grapheme_idx_to_col_idx(0), 0);
        assert_eq!(line.grapheme_idx_to_col_idx(3), 3);
        let line = Line::from("こんにちは");
        assert_eq!(line.col_idx_to_grapheme_idx(0), 0);
        assert_eq!(line.col_idx_to_grapheme_idx(4), 2);
        assert_eq!(line.grapheme_idx_to_col_idx(0), 0);
        assert_eq!(line.grapheme_idx_to_col_idx(2), 4);
    }

    #[test]
    fn test_split_off() {
        let mut line = Line::from("qwert");
        assert_eq!(line.content(), "qwert");
        let remainder = line.split_off(3);
        assert_eq!(line.content(), "qwe");
        assert_eq!(remainder.content(), "rt");
        let remainder = line.split_off(1);
        assert_eq!(line.content(), "q");
        assert_eq!(remainder.content(), "we");
        let remainder = line.split_off(1);
        assert_eq!(line.content(), "q");
        assert_eq!(remainder.content(), "");
        let remainder = line.split_off(0);
        assert_eq!(line.content(), "");
        assert_eq!(remainder.content(), "q");
    }
}
