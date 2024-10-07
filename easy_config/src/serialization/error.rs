use std::char::ParseCharError;
use std::fmt::{Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;
use crate::location::Location;
use crate::parser::parser_error::ParserError;

#[derive(Debug)]
pub enum Kind {
    ExpectedPresenceGot(String),
    ExpectedCollectionGot(String),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    UnableToFindKey(String),
    ExpectedTypeGot(String, String),
    ParseBoolError(ParseBoolError),
    ParseCharError(ParseCharError),
    WrongNumberOfElements(usize, usize),
    ParserError(ParserError),
    IoError(std::io::Error)
}

impl From<ParseIntError> for Kind {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl From<ParseFloatError> for Kind {
    fn from(value: ParseFloatError) -> Self {
        Self::ParseFloatError(value)
    }
}

impl From<ParseBoolError> for Kind {
    fn from(value: ParseBoolError) -> Self {
        Self::ParseBoolError(value)
    }
}

impl From<ParseCharError> for Kind {
    fn from(value: ParseCharError) -> Self {
        Self::ParseCharError(value)
    }
}

impl From<ParserError> for Kind {
    fn from(value: ParserError) -> Self {
        Self::ParserError(value)
    }
}
impl From<std::io::Error> for Kind {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

#[derive(Debug)]
pub struct Error {
    location: Option<Location>,
    kind: Kind
}

impl Error {
    pub fn at(kind: impl Into<Kind>, location: Option<Location>) -> Error {
        Self {
            location,
            kind: kind.into(),
        }
    }
}

impl From<ParserError> for Error {
    fn from(value: ParserError) -> Self {
        Self::at(value, None)
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::at(value, None)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let loc_string = if let Some(loc)  = self.location {
            format!(" @ ({}, {})", loc.row, loc.column)
        } else {
            String::new()
        };

        write!(f, "{}{}", match &self.kind {
            Kind::ExpectedPresenceGot(e) => format!("Expected a `Presence` but got: {}", e),
            Kind::ExpectedCollectionGot(e) => format!("Expected a `Collection` but got: {}", e),
            Kind::ParseIntError(e) => e.to_string(),
            Kind::ParseFloatError(e) => e.to_string(),
            Kind::UnableToFindKey(k) => k.to_string(),
            Kind::ExpectedTypeGot(t, k) => format!("Expected a `{}` but found: {}.", t, k),
            Kind::WrongNumberOfElements(g, e) => format!("Wrong number of elements. Expected {} got {}.", e, g),
            Kind::ParseBoolError(e) => e.to_string(),
            Kind::ParseCharError(e) => e.to_string(),
            Kind::ParserError(e) => e.to_string(),
            Kind::IoError(e) => e.to_string()
        }, loc_string)
    }
}

impl std::error::Error for Error {}