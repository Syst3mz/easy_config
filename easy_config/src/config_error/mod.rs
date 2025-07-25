pub mod describe;

use std::fmt::Display;
use crate::config_error::describe::Describe;
use crate::lexical_span::LexicalSpan;

pub const ERROR_WINDOW_SIZE: usize = 10;
fn build_error_area(span: LexicalSpan, source_text: impl AsRef<str>) -> String {
    let source_text = source_text.as_ref();
    
    let lowest_bound = span.start().saturating_sub(ERROR_WINDOW_SIZE);
    let left_dots = if lowest_bound > 0 { "..." } else { "" };

    let highest_bound = (span.end() + ERROR_WINDOW_SIZE).min(source_text.len());
    let right_dots = if highest_bound < source_text.len() {"..."} else {""};
    let index_of_offender = span.start() - lowest_bound;
    let mut offset = " ".repeat(index_of_offender);
    
    let span = LexicalSpan::new(lowest_bound, highest_bound);
    let window = span.slice(source_text);
    if window.is_empty() {
        return String::new()
    }
    offset.push('^');
    format!(
        "{}{}{}\n{}",
        left_dots,
        window,
        right_dots,
        offset
    )
}
#[derive(Debug, Clone)]
pub enum ConfigError<Kind> {
    FirstLevelError(Kind, String),
    ContextualizedError(String, Box<ConfigError<Kind>>)
}


impl<Kind> ConfigError<Kind> {
    pub fn on_span(kind: Kind, span: LexicalSpan, source_text: impl AsRef<str>) -> Self {
        let source_text = source_text.as_ref();
        let (row, col) = span.find_row_and_column(source_text);
        Self::FirstLevelError(kind, build_error_area(span, source_text))
            .contextualize(format!("Error at {row}:{col}"))
    }

    pub fn contextualize(self, context: impl AsRef<str>) -> Self {
        Self::ContextualizedError(context.as_ref().to_string(), Box::new(self))
    }
}

impl<Kind: Describe> Display for ConfigError<Kind> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ConfigError::FirstLevelError(kind, area) => {
                if area.is_empty() {
                    kind.describe()
                } else {
                    format!("{}\n{}", kind.describe(), area)
                }
            }
            ConfigError::ContextualizedError(context, err) =>
                format!("{}\n{}", context, err.to_string())
        })
    }
}

pub trait Contextualize {
    fn contextualize(self, context: impl AsRef<str>) -> Self;
}
impl<T, Kind> Contextualize for Result<T, ConfigError<Kind>> {
    fn contextualize(self, context: impl AsRef<str>) -> Self {
        self.map_err(|err| err.contextualize(context))
    }
}