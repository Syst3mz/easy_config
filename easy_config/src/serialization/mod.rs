mod primitives;
pub mod error;
mod tuples;

use std::path::Path;
use crate::expression::{CstData, CstExpression};
use crate::parser::Parser;
use crate::serialization::error::Error;
use crate::serialization::error::Kind::UnableToFindKey;

pub trait Config: 'static {
    fn serialize(&self) -> CstExpression;
    fn deserialize(expr: CstExpression) -> Result<Self, Error> where Self: Sized;
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum LoadMode {
    Loaded,
    Default
}

pub trait DefaultConfig: Config + Default {
    fn deserialize_from_file_or_default_and_write(path: impl AsRef<Path>) -> Result<(Self, LoadMode), Error> where Self: Sized {
        let path = path.as_ref();
        if std::fs::exists(path)? {
            Ok((Self::deserialize(Parser::parse(std::fs::read_to_string(path)?)?)?, LoadMode::Loaded))
        } else {
            let default = Self::default();

            std::fs::write(path, default.serialize().pretty())?;

            Ok((default, LoadMode::Default))
        }
    }
}

impl<T: Default + Config> DefaultConfig for T {}


type DeserializationIterator = std::vec::IntoIter<CstExpression>;

pub trait DeserializeExtension {
    fn deserialize_get(&self, key: impl AsRef<str>) -> Result<CstExpression, Error>;
    fn into_deserialization_iterator(self) -> Option<DeserializationIterator>;
}

impl DeserializeExtension for CstExpression {
    fn deserialize_get(&self, key: impl AsRef<str>) -> Result<CstExpression, Error> {
        let key = key.as_ref();
        self.get(key).ok_or(Error::at(UnableToFindKey(format!("Unable to find key \"{}\"", key)), self.location))
    }

    fn into_deserialization_iterator(self) -> Option<DeserializationIterator> {
        match self.data {
            CstData::Presence(_) => Some(vec![self].into_iter()),
            CstData::Pair(_, _) => None,
            CstData::Collection(c) => Some(c.into_iter())
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
        fn serialize(&self) -> CstExpression {
            CstExpression::collection(vec![
                CstExpression::pair("key".to_string(), self.key.serialize()),
                CstExpression::pair("vec".to_string(), self.vec.serialize())
            ])
        }

        fn deserialize(expr: CstExpression) -> Result<Self, Error> {
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

    #[should_panic]
    #[test]
    fn deserialize_err() {
        let parsed = Parser::new("(key = cat vec = a = b)").parse_tokens().unwrap();
        assert_eq!(
            Demo::deserialize(parsed).unwrap(), demo()
        )
    }
}