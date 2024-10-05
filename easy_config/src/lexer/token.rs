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

    pub fn kind(&self) -> Kind {
        self.kind
    }
    pub fn lexeme(&self) -> &String {
        &self.lexeme
    }
    pub fn row(&self) -> usize {
        self.row
    }
    pub fn column(&self) -> usize {
        self.column
    }
}