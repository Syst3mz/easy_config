pub mod primitives;
pub mod tuples;
pub mod serialization_error;
pub mod collections;
pub mod serialize_enum;

use std::path::Path;
use crate::expression::{ExpressionData, Expression};
use crate::parser::Parser;
use crate::serialization::serialization_error::{Kind, SerializationError};

pub trait Config: 'static {
    fn serialize(&self) -> Expression;
    fn deserialize(expr: Expression, source_text: impl AsRef<str>) -> Result<Self, SerializationError> where Self: Sized;
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum LoadMode {
    Loaded,
    Default
}

/// Any structure implementing both `Default` and `Config` automatically implements this trait.
pub trait DefaultConfig: Config + Default {
    /// This function always returns a config (if successful).
    /// If a configuration file is both present (at the specified path) and valid
    /// (can be interpreted without error), then that configuration is returned. If the config is
    /// not present, then a new file is generated at `path` and the default config is stored in it.
    /// If a config is invalid this function will return an error.
    fn deserialize_from_file_or_default_and_write(path: impl AsRef<Path>) -> Result<(Self, LoadMode), SerializationError> where Self: Sized {
        let path = path.as_ref();
        if std::fs::exists(path)? {
            let text = std::fs::read_to_string(path)?;
            let finished_parser = Parser::new(text.as_str()).parse();

            if !finished_parser.errors().is_empty() {
                return Err(SerializationError::FirstLevelError(Kind::ParserErrors(finished_parser.errors().clone()), String::new()));
            }

            let exprs  = finished_parser.expressions();


            Ok((Self::deserialize(exprs[0].clone(), text)?, LoadMode::Loaded))
        } else {
            let default = Self::default();

            std::fs::write(path, default.serialize().pretty())?;

            Ok((default, LoadMode::Default))
        }
    }
}

impl<T: Default + Config> DefaultConfig for T {}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::lexical_span::LexicalSpan;
    use crate::parser::Parser;
    use super::*;
    
    #[derive(Debug, PartialEq)]
    struct Demo {
        key: String,
        vec: Vec<i32>,
    }

    impl Config for Demo {
        fn serialize(&self) -> Expression {
            Expression::list(vec![
                self.key.serialize(),
                self.vec.serialize()
            ], LexicalSpan::zeros())
        }

        fn deserialize(expr: Expression, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
        where
            Self: Sized,
        {
            let span = expr.span();
            let source_text = source_text.as_ref();

            let mut iter = expr.into_iter();
            Ok(Self {
                key: String::deserialize(iter.next_field("key", source_text)?, source_text)?,
                vec: Vec::deserialize(iter.next_field("vec", source_text)?, source_text)?,
            })
        }
    }


    fn demo() -> Demo {
        Demo {
            key: "cat".to_string(),
            vec: vec![1, 2, 3],
        }
    }

    const EXPECTED: &'static str = "(cat (1 2 3))";
    #[test]
    fn serialize() {
        let d = demo();
        assert_eq!(d.serialize().minimized().dump(), EXPECTED)
    }

    #[test]
    fn deserialize() {
        let mut parsed = Parser::new(EXPECTED).parse().unwrap();
        assert_eq!(
            Demo::deserialize(parsed.remove(0), EXPECTED).unwrap(), demo()
        )
    }

    #[should_panic]
    #[test]
    fn deserialize_err() {
        let mut parsed = Parser::new("()").parse().unwrap();
        assert_eq!(
            Demo::deserialize(parsed.remove(0), "()").unwrap(), demo()
        )
    }
}