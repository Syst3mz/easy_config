use std::fmt::{Display, Formatter};
use std::io;
use itertools::Itertools;
use crate::location::Location;
use crate::span::Span;
use crate::token::{Token, TokenKind};

pub trait WithContext {
    fn with_context(self, message: impl AsRef<str>) -> Self;
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Error {
    TokenizationError(Span),
    UnexpectedToken(TokenKind, &'static [TokenKind], Span),
    ReachedEoi,
    Contextual(String, Box<Error>),
    UnableToConvert(Token, String, Span),
}

fn expected_error_text(got: &TokenKind, expected: &[TokenKind]) -> String {
    let expected = if expected.len() == 1 {
        format!("a {:?}", expected[0])
    } else {
        format!("one of [{}]", expected.iter().map(|x| format!("{:?}", x)).join(", "))
    };

    format!("Expected {} but got {:?}", expected, got)
}

impl Error {

    pub fn error_text(&self, source: impl AsRef<str>) -> String {
        let source = source.as_ref();
        match self {
            Error::TokenizationError(_) => "Tokenization error".to_string(),
            Error::UnexpectedToken(got, expected, _) => expected_error_text(got, expected),
            Error::ReachedEoi => "Reached end of input while parsing".to_string(),
            Error::Contextual(m, c) => format!("{m}\n{}", c.error_text(source)),
            Error::UnableToConvert(from, to, _) => format!("Unable to convert {} to {}", from.resolve(source), to),
        }
    }

    pub fn span(&self) -> Option<Span> {
        match self {
            Error::TokenizationError(s) => Some(*s),
            Error::UnexpectedToken(_, _, s) => Some(*s),
            Error::ReachedEoi => None,
            Error::Contextual(_, e) => e.span(),
            Error::UnableToConvert(_, _, s) => Some(*s),
        }
    }

    pub fn unable_to_convert_token_to(token: Token, to: impl AsRef<str>) -> Error {
        Error::UnableToConvert(token, to.as_ref().to_string(), token.span)
    }

    pub fn to_textual_error(self, source: &str) -> TextualError {
        TextualError::from_error(self, source)
    }
}

impl WithContext for Error {
    fn with_context(self, message: impl AsRef<str>) -> Error {
        Self::Contextual(message.as_ref().to_string(), Box::new(self))
    }
}

impl<T> WithContext for Result<T, Error> {
    fn with_context(self, message: impl AsRef<str>) -> Result<T, Error> {
        self.map_err(|x| x.with_context(message))
    }
}

pub trait ToTextualErrorResult<T> {
    fn to_textual_error(self, source: &str) -> Result<T, TextualError>;
}

impl<T> ToTextualErrorResult<T> for Result<T, Error> {
    fn to_textual_error(self, source: &str) -> Result<T, TextualError> {
        self.map_err(|x| x.to_textual_error(source))
    }
}

#[derive(Debug)]
pub struct TextualError {
    pub message: String,
    pub location: Option<Location>,
}

impl TextualError {
    pub fn from_error(error: Error, source: &str) -> TextualError {
        let location = Location::from_span(error.span().unwrap_or(Span::new(source.len() - 1, source.len())), source);

        TextualError {
            message: error.error_text(source),
            location: Some(location),
        }
    }
}

impl Display for TextualError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, 
               "{} at {}", 
               self.message, 
               if let Some(location) = &self.location { 
                   location.to_string() 
               } else { 
                   "serde originated error".to_string()
               }
        )
    }
}

impl std::error::Error for TextualError {}
impl From<io::Error> for TextualError {
    fn from(error: io::Error) -> Self {
        Self {
            message: error.to_string(),
            location: None,
        }
    }
}
