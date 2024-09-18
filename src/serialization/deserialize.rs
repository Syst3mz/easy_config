use crate::parser::expression::Expression;
use crate::serialization::deserialization_iter::DeserializationIterator;
use crate::serialization::error::Error;

pub trait Deserialize {
    fn deserialize(expr: Expression) -> Result<Self, Error> where Self: Sized;
}

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
            Expression::Presence(_) => Some(DeserializationIterator::new(vec![self])),
            Expression::Pair(_, _) => None,
            Expression::Collection(c) => Some(DeserializationIterator::new(c))
        }
    }
}