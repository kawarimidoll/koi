use unicode_width::UnicodeWidthStr;

pub struct TextFragment {
    pub grapheme: String,
    pub width: usize,
}

impl TextFragment {
    pub fn new(grapheme: &str) -> Self {
        let width = if grapheme.width() <= 1 { 1 } else { 2 };
        Self {
            grapheme: String::from(grapheme),
            width,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        // normal character
        let f = TextFragment::new("a");
        assert_eq!(f.grapheme, "a");
        assert_eq!(f.width, 1);

        // full-width character
        let f = TextFragment::new("緑");
        assert_eq!(f.grapheme, "緑");
        assert_eq!(f.width, 2);
    }
}
