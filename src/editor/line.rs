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
            .graphemes(true)
            .map(|grapheme| TextFragment::new(grapheme))
            .collect()
    }
    pub fn content(&self) -> &str {
        &self.string
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
    }
}
