use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use crate::span::Span;

#[derive(Debug)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Location { line, column }
    }

    pub fn from_span(span: impl Borrow<Span>, source: &str) -> Self {
        let span = span.borrow();
        let mut line = 1;
        let mut column = 0;

        for char in source.chars().take(span.start + 1) {
            if char == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }
        
        Location { line, column }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}