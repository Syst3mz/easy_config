use std::fmt::Display;
use itertools::Itertools;
use crate::expression::Expression;
use crate::lexer::token::Token;
use crate::lexical_range::LexicalSpan;

const WINDOW_SIZE: usize = 10;
type Tk = crate::lexer::token::Kind;

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

impl Kind {
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

fn build_error_area(range: LexicalSpan, source_text: impl AsRef<str>) -> String {
    // todo include line / col numbers
    let source_text = source_text.as_ref();
    let lowest_bound = range.start().saturating_sub(WINDOW_SIZE);
    let left_dots = if lowest_bound > 0 { "..." } else { "" };

    let highest_bound = (range.end() + WINDOW_SIZE).min(source_text.len());
    let right_dots = if highest_bound < source_text.len() {"..."} else {""};
    let index_of_offender = range.start() - lowest_bound;
    let mut offset = " ".repeat(index_of_offender);
    offset.push('^');
    format!(
        "{}{}{}\n{}",
        left_dots,
        &source_text[lowest_bound..highest_bound],
        right_dots,
        offset
    )
}
#[derive(Debug, Clone)]
pub enum ParserError {
    FirstLevelError(Kind, String),
    ContextualizedError(String, Box<ParserError>)
}


impl ParserError {
    pub fn on_span(kind: Kind, span: LexicalSpan, source_text: impl AsRef<str>) -> Self {
        Self::FirstLevelError(kind, build_error_area(span, source_text))
    }

    pub fn end_of_input(source_text: impl AsRef<str>) -> Self {
        let source_text = source_text.as_ref();
        let lowest_bound = source_text.len().saturating_sub(WINDOW_SIZE);
        let highest_bound = source_text.len() + lowest_bound;
        Self::FirstLevelError(Kind::ReachedEoi, build_error_area(LexicalSpan::new(lowest_bound, highest_bound), source_text))
    }

    pub fn contextualize(self, context: impl AsRef<str>) -> Self {
        Self::ContextualizedError(context.as_ref().to_string(), Box::new(self))
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ParserError::FirstLevelError(kind, area) => {
                format!("{}\n{}", kind.describe(), area)
            }
            ParserError::ContextualizedError(context, err) =>
                format!("{}\n{}", context, err.to_string())
        })
    }
}

pub trait Contextualize {
    fn contextualize(self, context: impl AsRef<str>) -> Self;
}
impl Contextualize for Result<Expression, ParserError> {
    fn contextualize(self, context: impl AsRef<str>) -> Self {
        self.map_err(|err| err.contextualize(context))
    }
}