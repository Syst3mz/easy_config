use std::any::type_name;
use crate::config::{Config, DeserializeExtension};
use crate::config::error::Error;
use crate::config::error::Error::{ExpectedCollectionGot, ExpectedTypeGot};
use crate::parser::expression::Expression;

impl<T1: Config, T2: Config> Config for (T1, T2) {
    fn serialize(&self) -> Expression {
        Expression::Collection(vec![self.0.serialize(), self.1.serialize()])
    }

    fn deserialize(expr: Expression) -> Result<Self, Error>
    where
        Self: Sized
    {
        let pretty = expr.pretty();
        let mut x = expr.into_deserialization_iterator()
            .ok_or(ExpectedCollectionGot(pretty.clone()))?;
        Ok((
            T1::deserialize(x.next().ok_or(ExpectedTypeGot(type_name::<T1>().to_string(), pretty.clone()))?)?,
            T2::deserialize(x.next().ok_or(ExpectedTypeGot(type_name::<T2>().to_string(), pretty))?)?,
        ))
    }
}