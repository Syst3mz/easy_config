use std::char::ParseCharError;
use std::fmt::{Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;
use crate::parser::parser_error::ParserError;

#[derive(Debug)]
pub enum Error {
    ExpectedPresenceGot(String),
    ExpectedCollectionGot(String),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    UnableToFindKey(String),
    ExpectedTypeGot(String, String),
    ParseBoolError(ParseBoolError),
    ParseCharError(ParseCharError),
    WrongNumberOfElements(usize, usize),
    ParserError(ParserError)
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl From<ParseFloatError> for Error {
    fn from(value: ParseFloatError) -> Self {
        Self::ParseFloatError(value)
    }
}

impl From<ParseBoolError> for Error {
    fn from(value: ParseBoolError) -> Self {
        Self::ParseBoolError(value)
    }
}

impl From<ParseCharError> for Error {
    fn from(value: ParseCharError) -> Self {
        Self::ParseCharError(value)
    }
}

impl From<ParserError> for Error {
    fn from(value: ParserError) -> Self {
        Self::ParserError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Error::ExpectedPresenceGot(e) => format!("Expected a `Presence` but got: {}", e),
            Error::ExpectedCollectionGot(e) => format!("Expected a `Collection` but got: {}", e),
            Error::ParseIntError(e) => e.to_string(),
            Error::ParseFloatError(e) => e.to_string(),
            Error::UnableToFindKey(k) => k.to_string(),
            Error::ExpectedTypeGot(t, k) => format!("Expected a `{}` but found: {}.", t, k),
            Error::WrongNumberOfElements(g, e) => format!("Wrong number of elements. Expected {} got {}.", e, g),
            Error::ParseBoolError(e) => e.to_string(),
            Error::ParseCharError(e) => e.to_string(),
            Error::ParserError(e) => e.to_string(),
        })
    }
}