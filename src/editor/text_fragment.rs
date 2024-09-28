use unicode_width::UnicodeWidthStr;

pub struct TextFragment {
    pub grapheme: String,
    width: usize,
    pub left_col_width: usize,
    replacement: Option<String>,
}

impl TextFragment {
    pub fn new(grapheme: &str, left_col_width: usize) -> Self {
        let replacement = Self::get_replacement(grapheme);
        let width = if replacement.is_some() {
            replacement.as_ref().unwrap().width()
        } else if grapheme.width() <= 1 {
            1
        } else {
            2
        };
        Self {
            grapheme: String::from(grapheme),
            width,
            left_col_width,
            replacement,
        }
    }
    fn get_replacement(grapheme: &str) -> Option<String> {
        let g_width = grapheme.width();
        match grapheme {
            " " => None,
            "\t" => Some("→".to_string()),
            _ if g_width > 0 && grapheme.trim().is_empty() => Some("␣".to_string()),
            _ if g_width == 0 => Some("·".to_string()),
            _ => {
                let mut chars = grapheme.chars();
                if let Some(ch) = chars.next() {
                    if ch.is_control() && chars.next().is_none() {
                        let replacement = ((ch as u8) + 64) as char;
                        return Some(format!("^{replacement}").to_string());
                    }
                }
                None
            }
        }
    }
    pub fn width(&self) -> usize {
        self.width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        // normal character
        let f = TextFragment::new("a", 0);
        assert_eq!(f.grapheme, "a");
        assert_eq!(f.width(), 1);
        assert_eq!(f.replacement, None);

        // full-width character
        let f = TextFragment::new("緑", 0);
        assert_eq!(f.grapheme, "緑");
        assert_eq!(f.width(), 2);
        assert_eq!(f.replacement, None);

        // tab
        let f = TextFragment::new("\t", 0);
        assert_eq!(f.grapheme, "\t");
        assert_eq!(f.width(), 1);
        assert_eq!(f.replacement, Some("→".to_string()));

        // ctrl character
        let f = TextFragment::new("\x01", 0);
        assert_eq!(f.grapheme, "\x01");
        assert_eq!(f.width(), 2);
        assert_eq!(f.replacement, Some("^A".to_string()));
    }
}
