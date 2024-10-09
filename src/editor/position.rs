use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Position {
    pub line_idx: usize,
    pub col: usize,
}
impl Position {
    #[cfg(test)]
    pub fn new(line_idx: usize, col: usize) -> Self {
        Self { line_idx, col }
    }
    pub const fn saturating_sub(&self, other: &Self) -> Self {
        Self {
            line_idx: self.line_idx.saturating_sub(other.line_idx),
            col: self.col.saturating_sub(other.col),
        }
    }
}
impl fmt::Display for Position {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "({}, {})", self.line_idx, self.col)
    }
}
