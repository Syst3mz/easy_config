pub mod describe;

use std::fmt::{Debug, Formatter};
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
    let mut offset = " ".repeat(index_of_offender + left_dots.len());
    
    let span = LexicalSpan::new(lowest_bound, highest_bound);
    let window = span.slice(source_text).replace('\n', " ");
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
#[derive(Clone)]
pub enum ConfigError<Kind> {
    FirstLevelError(Kind, LexicalSpan),
    ContextualizedError(String, Box<ConfigError<Kind>>)
}

impl<T: Debug> Debug for ConfigError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            ConfigError::FirstLevelError(kind, at) => format!("{:?} @ {}", kind, at),
            ConfigError::ContextualizedError(ctx, inner) => format!("{:?}\n{}", inner, ctx)
        })
    }
}

impl<Kind> ConfigError<Kind> {
    pub fn on_span(kind: impl Into<Kind>, span: LexicalSpan) -> Self {
        Self::FirstLevelError(kind.into(), span)
    }

    pub fn contextualize(self, context: impl AsRef<str>) -> Self {
        Self::ContextualizedError(context.as_ref().to_string(), Box::new(self))
    }
    
    pub fn kind(&self) -> &Kind {
        match self {
            ConfigError::FirstLevelError(k, _) => k,
            ConfigError::ContextualizedError(_, n) => n.kind(),
        }
    }
}

impl<Kind: Describe> ConfigError<Kind> {
    pub fn to_error_string(self, source_text: impl AsRef<str>) -> String {
        let source_text = source_text.as_ref();
        match self {
            ConfigError::FirstLevelError(k, s) => format!("{}\n{}", k.describe(source_text), build_error_area(s, source_text)),
            ConfigError::ContextualizedError(ctx, inner) => format!("{}\n{}", inner.to_error_string(&source_text), ctx)
        }
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