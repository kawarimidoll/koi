use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}
impl Position {
    #[cfg(test)]
    pub fn new(col: usize, row: usize) -> Self {
        Self { col, row }
    }
    pub const fn saturating_sub(&self, other: &Self) -> Self {
        Self {
            col: self.col.saturating_sub(other.col),
            row: self.row.saturating_sub(other.row),
        }
    }
}
impl fmt::Display for Position {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "({}, {})", self.col, self.row)
    }
}
