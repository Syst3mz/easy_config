use std::fmt::{Display, Formatter};
use crate::lexer::token::Token;
use crate::parser::Parser;

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    UnexpectedEquals,
    UnexpectedRParen,
    ExpectedLParen,
    ExpectedText,
    EOI
}
#[derive(Debug, Clone, Copy)]
pub struct ParserError {
    row: usize,
    column: usize,
    kind: ErrorKind
}
impl ParserError {
    fn on(token: &Token, kind: ErrorKind) -> Self {
        Self {
            row: token.row(),
            column: token.column(),
            kind,
        }
    }
    pub fn unexpected_equals(token: &Token) -> ParserError {
        Self::on(token, ErrorKind::UnexpectedEquals)
    }
    pub fn unexpected_r_paren(token: &Token) -> ParserError {
        Self::on(token, ErrorKind::UnexpectedRParen)
    }
    pub fn expected_l_paren(token: &Token) -> ParserError {
        Self::on(token, ErrorKind::ExpectedLParen)
    }
    pub fn expected_text(token: &Token) -> ParserError {
        Self::on(token, ErrorKind::ExpectedText)
    }

    pub fn eoi(parser: &Parser) -> ParserError {
        Self::on(&parser.tokens[parser.tokens.len() - 1], ErrorKind::EOI)
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let reason = match self.kind {
            ErrorKind::UnexpectedEquals => "unexpected \"=\"",
            ErrorKind::UnexpectedRParen => "unexpected \")\"",
            ErrorKind::ExpectedLParen => "expected \"(\"",
            ErrorKind::ExpectedText => "expected text that was not one of: \"=\", \")\", or \"(\"",
            ErrorKind::EOI => "reached end of input while parsing"
        };

        write!(f, "{} at {}:{}", reason, self.row, self.column)
    }
}

impl std::error::Error for ParserError {

}