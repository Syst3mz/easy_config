mod primitives;
mod tuples;
mod serialization_error;

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


type DeserializationIterator = std::vec::IntoIter<Expression>;

pub trait DeserializeExtension {
    fn deserialize_get(&self, key: impl AsRef<str>) -> Result<Expression, Kind>;
    fn into_deserialization_iterator(self) -> Option<DeserializationIterator>;
}

impl DeserializeExtension for Expression {
    fn deserialize_get(&self, key: impl AsRef<str>) -> Result<Expression, Kind> {
        let key = key.as_ref();
        self.get(key).ok_or(Kind::UnableToLocateBindingName(key.to_string()))
    }

    fn into_deserialization_iterator(self) -> Option<DeserializationIterator> {
        match self.data {
            ExpressionData::Presence(_, _) | ExpressionData::Binding(_, _, _) => Some(vec![self].into_iter()),
            ExpressionData::List(c, _) => Some(c.into_iter())
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::lexical_range::LexicalSpan;
    use crate::parser::Parser;
    use super::*;

    #[derive(Debug, PartialEq)]
    enum DemoEnum {
        Hi,
        String(String)
    }
    
    #[derive(Debug, PartialEq)]
    struct Demo {
        key: String,
        vec: Vec<DemoEnum>,
    }

    impl Config for Demo {
        fn serialize(&self) -> Expression {
            let list: Vec<Expression> = self.vec.iter().map(|x| match x {
                DemoEnum::Hi => Expression::presence("Hi", LexicalSpan::zeros()),
                DemoEnum::String(s) => Expression::list(vec![
                    Expression::presence("String", LexicalSpan::zeros()),
                    Expression::list(vec![Expression::presence(s.as_str(), LexicalSpan::zeros())], LexicalSpan::zeros())
                ], LexicalSpan::zeros()),
            }).collect();
            Expression::list(vec![
                Expression::binding("key".into(), Expression::presence(self.key.clone(), LexicalSpan::zeros()), LexicalSpan::zeros()),
                Expression::list(list, LexicalSpan::zeros()),
            ], LexicalSpan::new(0,0,))
        }

        fn deserialize(expr: Expression, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
        where
            Self: Sized,
        {
            todo!()
        }
    }


    fn demo() -> Demo {
        Demo {
            key: "cat".to_string(),
            vec: vec![DemoEnum::Hi, DemoEnum::String("dog".to_string())],
        }
    }

    #[test]
    fn serialize() {
        let d = demo();
        assert_eq!(d.serialize().dump(), "(key = cat vec = (bird dog))")
    }

    /*#[test]
    fn deserialize() {
        let parsed = Parser::new(demo().serialize().dump()).parse().unwrap();
        assert_eq!(
            Demo::deserialize(parsed).unwrap(), demo()
        )
    }

    #[should_panic]
    #[test]
    fn deserialize_err() {
        let parsed = Parser::new("(key = cat vec = a = b)").parse().unwrap();
        assert_eq!(
            Demo::deserialize(parsed).unwrap(), demo()
        )
    }*/
}