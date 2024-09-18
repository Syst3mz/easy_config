use std::fmt::{Display, Formatter};
use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug)]
pub enum Error {
    ExpectedPresenceGot(String),
    ExpectedCollectionGot(String),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
    UnableToFindKey(String),
    ExpectedTypeGot(String, String)
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

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Error::ExpectedPresenceGot(e) => format!("Expected a `Presence` but got: {}", e),
            Error::ExpectedCollectionGot(e) => format!("Expected a `Collection` but got: {}", e),
            Error::ParseIntError(e) => e.to_string(),
            Error::ParseFloatError(e) => e.to_string(),
            Error::UnableToFindKey(k) => k.to_string(),
            Error::ExpectedTypeGot(t, k) => format!("Expected a `{}` but found: {}.", t, k)
        })
    }
}