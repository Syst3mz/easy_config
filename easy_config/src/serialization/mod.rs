pub mod primitives;
pub mod tuples;
pub mod serialization_error;
pub mod option_span_combine;
mod option;
mod collections;

use std::path::Path;
use anyhow::anyhow;
use crate::expression::Expression;
use crate::expression_iterator::ExpressionIterator;
use crate::lexical_span::LexicalSpan;
use crate::parser::Parser;
use crate::serialization::serialization_error::{Kind, SerializationError};

pub trait EasyConfig: 'static {

    const IS_ENUM: bool = false;
    const IS_STRUCT: bool = false;

    /// Is composite tells Easy Config if a type is composed of other types (enums are considered a composite type) AND has a name.
    fn is_composite() -> bool { Self::IS_ENUM || Self::IS_STRUCT }

    fn serialize(&self) -> Expression;
    fn deserialize(expression_iterator: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError> where Self: Sized;
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum LoadMode {
    Loaded,
    Default
}

/// Any structure implementing both `Default` and `Config` automatically implements this trait.
pub trait DefaultConfig: EasyConfig + Default {
    /// This function always returns a config (if successful).
    /// If a configuration file is both present (at the specified path) and valid
    /// (can be interpreted without error), then that configuration is returned. If the config is
    /// not present, then a new file is generated at `path` and the default config is stored in it.
    /// If a config is invalid this function will return an error.
    fn deserialize_from_file_or_default_and_write(path: impl AsRef<Path>) -> anyhow::Result<(Self, LoadMode)> where Self: Sized {
        let path = path.as_ref();
        if std::fs::exists(path)? {
            let text = std::fs::read_to_string(path)?;
            let finished_parser = Parser::new(text.as_str()).parse();

            if !finished_parser.errors().is_empty() {
                let err = SerializationError::FirstLevelError(Kind::ParserErrors(finished_parser.errors().clone()), LexicalSpan::zeros());
                return Err(anyhow!(err.to_error_string(&text)));
            }

            let expr  = finished_parser.unwrap();
            let x = Self::deserialize(&mut expr.into_iter(), &text).map_err(|err| anyhow!(err.to_error_string(&text)))?;
            Ok((x, LoadMode::Loaded))
        } else {
            let default = Self::default();

            std::fs::write(path, default.serialize().pretty())?;

            Ok((default, LoadMode::Default))
        }
    }
}

impl<T: Default + EasyConfig> DefaultConfig for T {}