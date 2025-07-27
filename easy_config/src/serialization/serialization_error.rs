use std::char::ParseCharError;
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;
use crate::config_error::ConfigError;
use crate::expression::Expression;
use crate::parser::parser_error::ParserError;

pub type SerializationError = ConfigError<Kind>;
#[derive(Debug)]
pub enum Kind {
    ParserErrors(Vec<ParserError>),
    FileError(std::io::Error),
    UnableToLocateBindingName(String),
    WrongCardinality { got: usize, want: usize },

    ParseIntError(ParseIntError),
    ParseCharError(ParseCharError),
    ParseFloatError(ParseFloatError),
    ParseBoolError(ParseBoolError),

    ExpectedNumber(String),
    ExpectedText(String),
    ExpectedPresence(Expression),
    ExpectedBinding(Expression),
    ExpectedList(Expression),

    ExpectedDiscriminant(String, &'static [&'static str]),
    MissingField(String),

    ExpectedFieldGotEoi(String),
    ReachedEoi
}

impl From<std::io::Error> for SerializationError {
    fn from(value: std::io::Error) -> Self {
        SerializationError::FirstLevelError(Kind::FileError(value), String::new())
    }
}
impl From<ParseIntError> for SerializationError {
    fn from(value: ParseIntError) -> Self {
        SerializationError::FirstLevelError(Kind::ParseIntError(value), String::new())
    }
}
impl From<ParseCharError> for SerializationError {
    fn from(value: ParseCharError) -> Self {
        SerializationError::FirstLevelError(Kind::ParseCharError(value), String::new())
    }
}
impl From<ParseFloatError> for SerializationError {
    fn from(value: ParseFloatError) -> Self {
        SerializationError::FirstLevelError(Kind::ParseFloatError(value), String::new())
    }
}
impl From<ParseBoolError> for SerializationError {
    fn from(value: ParseBoolError) -> Self {
        SerializationError::FirstLevelError(Kind::ParseBoolError(value), String::new())
    }
}