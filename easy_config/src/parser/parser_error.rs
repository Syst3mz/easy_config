use std::fmt::Display;
use itertools::Itertools;
use crate::config_error::{ConfigError, ERROR_WINDOW_SIZE};
use crate::config_error::describe::Describe;
use crate::expression::Expression;
use crate::lexer::token::Token;
use crate::lexical_range;
use crate::lexical_range::LexicalSpan;

type Tk = crate::lexer::token::Kind;
pub type ParserError = ConfigError<Kind>;

#[derive(Debug, Clone)]
pub enum Kind {
    UnexpectedToken(Token, &'static [Tk]),
    ReachedEoi,
    InvalidIdentifier(Token),
}

fn unexpected_text<G: ToString + ?Sized, E: ToString>(got: &G, expected: &[E]) -> String {
    let expected = if expected.len() == 1 {
        format!("Expected a {}", expected[0].to_string())
    } else {
        format!("Expected one of [{}]", expected.iter().map(|x| x.to_string()).join(", "))
    };

    format!("Unexpected token '{}'. {}.", got.to_string(), expected)
}
impl Describe for Kind {
    fn describe(&self) -> String {
        match self {
            Kind::UnexpectedToken(token, expected) => unexpected_text(
                token.lexeme(), expected
            ),
            Kind::ReachedEoi => String::from("Reached end of input while parsing!"),
            Kind::InvalidIdentifier(token) => format!("Invalid identifier '{}'.", token.lexeme()),
        }
    }
}

pub fn end_of_input(source_text: impl AsRef<str>) -> ParserError {
    let source_text = source_text.as_ref();
    let span = LexicalSpan::new(source_text.len() - 1, source_text.len());
    ParserError::on_span(Kind::ReachedEoi, span, source_text)
}