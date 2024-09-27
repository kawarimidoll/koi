use super::text_fragment::TextFragment;
use std::{fmt, ops::Deref};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Line {
    fragments: Vec<TextFragment>,
    string: String,
}

#[allow(dead_code)]
impl Line {
    pub fn from(string: &str) -> Self {
        debug_assert!(string.is_empty() || string.lines().count() == 1);
        Self {
            fragments: Self::string_to_fragments(string),
            string: String::from(string),
        }
    }
    fn string_to_fragments(string: &str) -> Vec<TextFragment> {
        string
            .grapheme_indices(true)
            .map(|(left_width, grapheme)| TextFragment::new(grapheme, left_width))
            .collect()
    }
    pub fn content(&self) -> &str {
        &self.string
    }
    pub fn grapheme_count(&self) -> usize {
        self.fragments.len()
    }
    pub fn get_fragment_by_byte_idx(&self, byte_idx: usize) -> Option<&TextFragment> {
        let mut acc = 0;
        for fragment in self.fragments.iter() {
            acc += fragment.width;
            if acc > byte_idx {
                return Some(fragment);
            }
        }
        None
    }
    pub fn grapheme_idx_to_byte_idx(&self, grapheme_idx: usize) -> usize {
        self.fragments
            .iter()
            .take(grapheme_idx)
            .map(|fragment| fragment.width)
            .sum()
    }
    pub fn len(&self) -> usize {
        self.string.len()
    }
}

impl fmt::Display for Line {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "{}", self.string)
    }
}
impl Deref for Line {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.string
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
        assert_eq!(line.grapheme_idx_to_byte_idx(5), 5);
        assert_eq!(
            line.get_fragment_by_byte_idx(4).map_or("", |f| &f.grapheme),
            "_"
        );
        assert_eq!(
            line.get_fragment_by_byte_idx(5).map_or("", |f| &f.grapheme),
            "f"
        );

        let line = Line::from("こんにちは");
        assert_eq!(line.content(), "こんにちは");
        assert_eq!(line.grapheme_count(), 5);
        assert_eq!(line.grapheme_idx_to_byte_idx(2), 4);
        assert_eq!(
            line.get_fragment_by_byte_idx(4).map_or("", |f| &f.grapheme),
            "に"
        );
        assert_eq!(
            line.get_fragment_by_byte_idx(5).map_or("", |f| &f.grapheme),
            "に"
        );
    }
}
