#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Location {
    pub row: usize,
    pub column: usize
}

impl Location {
    pub fn new(row: usize, column: usize) -> Self {
        Self {
            row,
            column,
        }
    }
}