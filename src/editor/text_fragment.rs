pub struct TextFragment {
    pub grapheme: String,
}

impl TextFragment {
    pub fn new(grapheme: &str) -> Self {
        Self {
            grapheme: String::from(grapheme),
        }
    }
}
