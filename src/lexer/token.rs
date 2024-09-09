#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Kind {
    LParen,
    RParen,
    Text,
    Equals
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Token {
    kind: Kind,
    lexeme: String,
    row: usize,
    column: usize,
}

impl Token {
    pub fn new(kind: Kind, lexeme: impl AsRef<str>, row: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme: lexeme.as_ref().to_string(),
            row,
            column,
        }
    }
}