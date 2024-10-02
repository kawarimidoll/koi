use super::text_fragment::TextFragment;
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

// https://rust-lang.github.io/rust-clippy/master/index.html#/format_collect
use std::fmt::Write;

const ELLIPSIS_LEFT: &str = "«";
const ELLIPSIS_RIGHT: &str = "»";

#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
    string: String,
    col_width: usize,
}

#[allow(dead_code)]
impl Line {
    pub fn from(string: &str) -> Self {
        debug_assert!(string.is_empty() || string.lines().count() == 1);
        let (fragments, col_width) = Self::string_to_fragments(string);
        Self {
            fragments,
            string: String::from(string),
            col_width,
        }
    }
    fn string_to_fragments(string: &str) -> (Vec<TextFragment>, usize) {
        let mut left_width = 0;
        let fragments = string
            .graphemes(true)
            .map(|grapheme| {
                let fragment = TextFragment::new(grapheme, left_width);
                left_width = left_width.saturating_add(fragment.width());
                fragment
            })
            .collect();

        (fragments, left_width)
    }
    pub fn content(&self) -> &str {
        &self.string
    }
    pub fn grapheme_count(&self) -> usize {
        self.fragments.len()
    }
    pub fn get_str_by_col_range(&self, range: Range<usize>) -> String {
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
    pub fn col_width(&self) -> usize {
        self.col_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        let line = Line::from("test_from");
        assert_eq!(line.content(), "test_from");
        assert_eq!(line.grapheme_count(), 9);
        assert_eq!(line.col_width(), 9);
        assert_eq!(line.grapheme_idx_to_col_idx(5), 5);
        assert_eq!(
            line.get_fragment_by_col_idx(4).map_or("", |f| &f.grapheme),
            "_"
        );
        assert_eq!(
            line.get_fragment_by_col_idx(5).map_or("", |f| &f.grapheme),
            "f"
        );

        assert_eq!(line.get_str_by_col_range(2..6), "st_f");
        assert_eq!(line.get_str_by_col_range(1..5), "est_");

        let line = Line::from("こんにちは");
        assert_eq!(line.content(), "こんにちは");
        assert_eq!(line.grapheme_count(), 5);
        assert_eq!(line.col_width(), 10);
        assert_eq!(line.grapheme_idx_to_col_idx(2), 4);
        assert_eq!(
            line.get_fragment_by_col_idx(4).map_or("", |f| &f.grapheme),
            "に"
        );
        assert_eq!(
            line.get_fragment_by_col_idx(5).map_or("", |f| &f.grapheme),
            "に"
        );
        assert_eq!(line.get_str_by_col_range(2..6), "んに");
        assert_eq!(line.get_str_by_col_range(1..5), "«ん»");
    }
}
