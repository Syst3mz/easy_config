use std::any::type_name;
use std::collections::HashMap;
use std::hash::Hash;
use crate::parser::expression::{escape, Expression};
use crate::serialization::deserialize::{Deserialize, DeserializeExtension};
use crate::serialization::error::Error;
use crate::serialization::Serialize;

macro_rules! serialize {
    ($ty: ty) => {
        impl Serialize for $ty {
            fn serialize(&self) -> Expression {
                Expression::Presence(self.to_string())
            }
        }
    };
}

macro_rules! deserialize {
    ($ty: ty) => {
        impl Deserialize for $ty {
            fn deserialize(expr: Expression) -> Result<Self, Error> {
                match expr {
                    Expression::Presence(p) => Ok(p.parse()?),
                    _ => Err(Error::ExpectedPresenceGot(expr.pretty()))
                }
            }
        }
    };
}

macro_rules! impl_primitive {
    ($ty: ty) => {
        serialize!($ty);
        deserialize!($ty);
    };
}


impl_primitive!(i8);
impl_primitive!(i16);
impl_primitive!(i32);
impl_primitive!(i64);
impl_primitive!(i128);

impl_primitive!(u8);
impl_primitive!(u16);
impl_primitive!(u32);
impl_primitive!(u64);
impl_primitive!(u128);

impl_primitive!(f32);
impl_primitive!(f64);
serialize!(&'static str);
serialize!(str);

impl Serialize for String {
    fn serialize(&self) -> Expression {
        Expression::Collection(Vec::from_iter(
            escape(self).split(|x: char| x.is_whitespace()).filter_map(|x| if !x.is_empty() {Some(x.serialize())} else {None})
        )).minimized()
    }
}
impl Deserialize for String {
    fn deserialize(expr: Expression) -> Result<Self, Error>
    where
        Self: Sized
    {
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

impl<T: Serialize> Serialize for [T] {
    fn serialize(&self) -> Expression {
        Expression::Collection(Vec::from_iter(self.iter().map(|x| x.serialize())))
    }
}
impl<K: Serialize, V: Serialize> Serialize for HashMap<K, V> {
    fn serialize(&self) -> Expression {
        Expression::Collection(Vec::from_iter(self.iter().map(|(k, v)| Expression::Collection(vec![k.serialize(), v.serialize()]))))
    }
}
impl<K: Deserialize+Eq+Hash, V: Deserialize+Eq+Hash> Deserialize for HashMap<K, V> {
    fn deserialize(expr: Expression) -> Result<Self, Error>
    where
        Self: Sized
    {
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

impl<T: Deserialize> Deserialize for Vec<T> {
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

impl<T: Serialize> Serialize for Box<T> {
    fn serialize(&self) -> Expression {
        T::serialize(self)
    }
}

impl<T: Deserialize> Deserialize for Box<T> {
    fn deserialize(expr: Expression) -> Result<Self, Error>
    where
        Self: Sized
    {
        Ok(Box::new(T::deserialize(expr)?))
    }
}

