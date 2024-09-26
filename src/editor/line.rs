use std::{fmt, ops::Deref};

#[derive(Eq, PartialEq, Debug, Default)]
pub struct Line {
    string: String,
}

#[allow(dead_code)]
impl Line {
    pub fn from(string: &str) -> Self {
        Self {
            string: String::from(string),
        }
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
