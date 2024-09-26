use std::any::type_name;
use std::collections::HashMap;
use std::hash::Hash;
use crate::parser::expression::{escape, Expression};
use crate::config::error::Error;
use crate::config::{Config, DeserializeExtension};
use crate::config::error::Error::WrongNumberOfElements;
use crate::parser::expression::Expression::Presence;

macro_rules! config {
    ($ty: ty) => {
        impl Config for $ty {
            fn serialize(&self) -> Expression {
                Expression::Presence(self.to_string())
            }

            fn deserialize(expr: Expression) -> Result<Self, Error> {
                match expr {
                    Expression::Presence(p) => Ok(p.parse()?),
                    _ => Err(Error::ExpectedPresenceGot(expr.pretty()))
                }
            }
        }
    };
}

config!(i8);
config!(i16);
config!(i32);
config!(i64);
config!(i128);

config!(u8);
config!(u16);
config!(u32);
config!(u64);
config!(u128);

config!(f32);
config!(f64);


impl Config for String {
    fn serialize(&self) -> Expression {
        Expression::Collection(Vec::from_iter(
            escape(self).split(|x: char| x.is_whitespace()).filter_map(|x| if !x.is_empty() { Some(Presence(x.to_string())) } else {None})
        )).minimized()
    }

    fn deserialize(expr: Expression) -> Result<Self, Error> {
        let words = expr
            .clone()
            .into_deserialization_iterator()
            .ok_or(Error::ExpectedTypeGot(type_name::<String>().to_string(), expr.pretty()))?;

        let mut acc = String::new();
        for word in words {
            acc += &word.clone().release().ok_or(Error::ExpectedPresenceGot(word.pretty()))?;
            acc.push(' ');
        }

        if !acc.is_empty() {
            acc.pop();
        }

        Ok(acc)
    }
}
impl <T: Config, const N: usize > Config for [T; N] {
    fn serialize(&self) -> Expression {
        Expression::Collection(Vec::from_iter(self.iter().map(|x| x.serialize())))
    }

    fn deserialize(expr: Expression) -> Result<Self, Error> {
        let elements = expr
            .clone()
            .into_deserialization_iterator()
            .ok_or(Error::ExpectedTypeGot(type_name::<Vec<T>>().to_string(), expr.pretty()))?;

        let maybe_good_size_store: Vec<T> = Result::from_iter(elements.map(|x| T::deserialize(x)))?;
        Ok(maybe_good_size_store.try_into().map_err(|x: Vec<T>| WrongNumberOfElements(x.len(), N))?)
    }
}
impl<T: Config> Config for Vec<T> {
    fn serialize(&self) -> Expression {
        Expression::Collection(Vec::from_iter(self.iter().map(|x| x.serialize())))
    }

    fn deserialize(expr: Expression) -> Result<Self, Error>
    where
        Self: Sized
    {
        let elements = expr
            .clone()
            .into_deserialization_iterator()
            .ok_or(Error::ExpectedTypeGot(type_name::<Vec<T>>().to_string(), expr.pretty()))?;

        Ok(Result::from_iter(elements.map(|x| T::deserialize(x)))?)
    }
}
impl<K: Config+Hash+Eq, V: Config> Config for HashMap<K, V> {
    fn serialize(&self) -> Expression {
        Expression::Collection(Vec::from_iter(self.iter().map(|(k, v)| Expression::Collection(vec![k.serialize(), v.serialize()]))))
    }

    fn deserialize(expr: Expression) -> Result<Self, Error> {
        let kv_pairs = expr
            .clone()
            .into_deserialization_iterator()
            .ok_or(Error::ExpectedCollectionGot(expr.pretty()))?;

        let mut hm = HashMap::new();

        for kv_pair in kv_pairs {
            let mut kv_pair_iter = kv_pair
                .clone()
                .into_deserialization_iterator()
                .ok_or(Error::ExpectedCollectionGot(kv_pair.pretty()))?;
            let k = kv_pair_iter
                .next()
                .ok_or(Error::ExpectedTypeGot(type_name::<K>().to_string(), kv_pair.pretty()))?;
            let k = K::deserialize(k)?;

            let v = kv_pair_iter
                .next()
                .ok_or(Error::ExpectedTypeGot(type_name::<V>().to_string(), kv_pair.pretty()))?;
            let v = V::deserialize(v)?;

            hm.insert(k, v);
        }

        Ok(hm)
    }
}
impl<T: Config> Config for Box<T> {
    fn serialize(&self) -> Expression {
        T::serialize(self)
    }

    fn deserialize(expr: Expression) -> Result<Self, Error> {
        Ok(Box::new(T::deserialize(expr)?))
    }
}

