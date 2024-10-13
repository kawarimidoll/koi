use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}
impl Size {
    #[cfg(test)]
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}
impl fmt::Display for Size {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "({}, {})", self.width, self.height)
    }
}
