use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Position {
    pub line_idx: usize,
    pub col_idx: usize,
}
impl Position {
    #[cfg(test)]
    pub fn new(line_idx: usize, col_idx: usize) -> Self {
        Self { line_idx, col_idx }
    }
    pub const fn saturating_sub(&self, other: &Self) -> Self {
        Self {
            line_idx: self.line_idx.saturating_sub(other.line_idx),
            col_idx: self.col_idx.saturating_sub(other.col_idx),
        }
    }
}
impl fmt::Display for Position {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "({}, {})", self.line_idx, self.col_idx)
    }
}
