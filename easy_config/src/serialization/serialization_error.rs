use std::char::ParseCharError;
use std::num::{ParseFloatError, ParseIntError};
use std::str::ParseBoolError;
use itertools::Itertools;
use crate::config_error::ConfigError;
use crate::config_error::describe::Describe;
use crate::expression::Expression;
use crate::lexical_span::LexicalSpan;
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

fn expected_from_options_text(options: &[impl AsRef<str>]) -> String {
    if options.len() == 1 {
        options[0].as_ref().to_string()
    } else {
        format!("one of [{}]", options.iter().map(|x| x.as_ref()).join(", "))
    }
}
impl Describe for Kind {
    fn describe(&self) -> String {
        match self {
            Kind::ParserErrors(errors) => format!("Unable to parse expression.\n{}", errors.iter().map(|x| x.to_string()).join("\n")),
            Kind::FileError(e) => format!("Unable to open file:\n{}", e),
            Kind::UnableToLocateBindingName(n) => format!("The binding {} is mandatory, but not present.", n),
            Kind::WrongCardinality { got, want } => format!("Wrong cardinality. Expected to have {} elements, but got {} elements", want, got),
            Kind::ParseIntError(e) => e.to_string(),
            Kind::ParseCharError(e) => e.to_string(),
            Kind::ParseFloatError(e) => e.to_string(),
            Kind::ParseBoolError(e) => e.to_string(),
            Kind::ExpectedNumber(s) => format!("Expected number, but got {}.", s),
            Kind::ExpectedText(s) => format!("Expected text, but got {}.", s),
            Kind::ExpectedPresence(g) => format!("Expected Presence, but got {}.", g.data.name_of_kind()),
            Kind::ExpectedBinding(g) => format!("Expected Binding, but got {}.", g.data.name_of_kind()),
            Kind::ExpectedList(g) => format!("Expected List, but got {}.", g.data.name_of_kind()),
            Kind::ExpectedDiscriminant(got, options) => format!("Expected a enum discriminant (specifically {}), but got {}.", expected_from_options_text(options), got),
            Kind::MissingField(f) => format!("Expected to find the field: {}.", f),
            Kind::ExpectedFieldGotEoi(e) => format!("Expected to find a field called {} but got to the end of the input.", e),
            Kind::ReachedEoi => "Reached the end of the file unexpectedly.".to_string()
        }
    }
}

impl SerializationError {
    pub fn end_of_input(source_text: impl AsRef<str>) -> SerializationError {
        let source_text = source_text.as_ref();
        let span = LexicalSpan::new(source_text.len() - 1, source_text.len());
        SerializationError::on_span(Kind::ReachedEoi, span, source_text)
    }
}