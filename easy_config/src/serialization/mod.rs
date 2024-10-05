mod primitives;
pub mod error;
mod tuples;

use std::path::Path;
use crate::expression::Expression;
use crate::parser::Parser;
use crate::serialization::error::Error;

pub trait Config: 'static {
    fn serialize(&self) -> Expression;
    fn deserialize(expr: Expression) -> Result<Self, Error> where Self: Sized;
}

pub trait DefaultConfig: Config + Default {
    fn deserialize_from_file_or_default(path: impl AsRef<Path>) -> Result<Self, Error> where Self: Sized {
        let path = path.as_ref();
        if std::fs::exists(path)? {
            Self::deserialize(Parser::parse(std::fs::read_to_string(path)?)?)
        } else {
            Ok(Self::default())
        }
    }
}

impl<T: Default + Config> DefaultConfig for T {}


type DeserializationIterator = std::vec::IntoIter<Expression>;

pub trait DeserializeExtension {
    fn deserialize_get(&self, key: impl AsRef<str>) -> Result<Expression, Error>;
    fn into_deserialization_iterator(self) -> Option<DeserializationIterator>;
}

impl DeserializeExtension for Expression {
    fn deserialize_get(&self, key: impl AsRef<str>) -> Result<Expression, Error> {
        let key = key.as_ref();
        self.get(key).ok_or(Error::UnableToFindKey(format!("Unable to find key \"{}\"", key)))
    }

    fn into_deserialization_iterator(self) -> Option<DeserializationIterator> {
        match self {
            Expression::Presence(_) => Some(vec![self].into_iter()),
            Expression::Pair(_, _) => None,
            Expression::Collection(c) => Some(c.into_iter())
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::parser::Parser;
    use crate::serialization::error::Error;
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Demo {
        key: String,
        vec: Vec<String>,
    }

    impl Config for Demo {
        fn serialize(&self) -> Expression {
            Expression::Collection(vec![
                Expression::Pair("key".to_string(), Box::new(self.key.serialize())),
                Expression::Pair("vec".to_string(), Box::new(self.vec.serialize()))
            ])
        }

        fn deserialize(expr: Expression) -> Result<Self, Error> {
            Ok(Self {
                key: String::deserialize(expr.deserialize_get("key")?)?,
                vec: Vec::<String>::deserialize(expr.deserialize_get("vec")?)?
            })
        }
    }


    fn demo() -> Demo {
        Demo {
            key: "cat".to_string(),
            vec: vec!["bird".to_string(), "dog".to_string()],
        }
    }

    #[test]
    fn serialize() {
        let d = demo();
        assert_eq!(d.serialize().dump(), "(key = cat vec = (bird dog))")
    }

    #[test]
    fn deserialize() {
        let parsed = Parser::new(demo().serialize().dump()).parse_tokens().unwrap();
        assert_eq!(
            Demo::deserialize(parsed).unwrap(), demo()
        )
    }
}