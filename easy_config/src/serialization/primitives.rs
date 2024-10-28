use std::any::type_name;
use std::collections::HashMap;
use std::hash::Hash;
use crate::expression::{escape, CstExpression, CstData};
use crate::expression::CstData::{Collection, Presence};
use crate::serialization::error::Error;
use crate::serialization::{Config, DeserializationIterator, DeserializeExtension};
use crate::serialization::error::Kind::{ExpectedCollectionGot, WrongNumberOfElements, ExpectedPresenceGot, ExpectedTypeGot};


macro_rules! config {
    ($ty: ty) => {
        impl Config for $ty {
            fn serialize(&self) -> CstExpression {
                CstExpression::presence(self.to_string())
            }

            fn deserialize(expr: CstExpression) -> Result<Self, Error> {
                match expr.data {
                    CstData::Presence(p) => Ok(p.parse().map_err(|x| Error::at(x, expr.location))?),
                    _ => Err(Error::at(ExpectedPresenceGot(expr.pretty()), expr.location))
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
config!(isize);

config!(u8);
config!(u16);
config!(u32);
config!(u64);
config!(u128);
config!(usize);

config!(f32);
config!(f64);

config!(bool);
config!(char);


fn deserialization_iter<T>(expr: CstExpression) -> Result<DeserializationIterator, Error> {
    let error_if_needed = Error::at(ExpectedTypeGot(type_name::<Vec<T>>().to_string(), expr.pretty()), expr.location);
    expr
        .into_deserialization_iterator()
        .ok_or(error_if_needed)
}

impl Config for String {
    fn serialize(&self) -> CstExpression {
        CstExpression::collection(Vec::from_iter(
            escape(self).split(|x: char| x.is_whitespace()).filter_map(|x| if !x.is_empty() { Some(Presence(x.to_string()).into()) } else {None})
        )).minimized()
    }

    fn deserialize(expr: CstExpression) -> Result<Self, Error> {
        let words = deserialization_iter::<Self>(expr.clone())?;

        let mut acc = String::new();
        for word in words {
            acc += &word.clone().release().ok_or(Error::at(ExpectedPresenceGot(word.pretty()), word.location))?;
            acc.push(' ');
        }

        if !acc.is_empty() {
            acc.pop();
        }

        Ok(acc)
    }
}
impl <T: Config, const N: usize > Config for [T; N] {
    fn serialize(&self) -> CstExpression {
        CstExpression::collection(Vec::from_iter(self.iter().map(|x| x.serialize())))
    }

    fn deserialize(expr: CstExpression) -> Result<Self, Error> {
        let elements = deserialization_iter::<Self>(expr.clone())?;

        let maybe_good_size_store: Vec<T> = Result::from_iter(elements.map(|x| T::deserialize(x)))?;
        Ok(maybe_good_size_store.try_into().map_err(|x: Vec<T>| Error::at(WrongNumberOfElements(x.len(), N), expr.location))?)
    }
}
impl<T: Config> Config for Vec<T> {
    fn serialize(&self) -> CstExpression {
        CstExpression::collection(Vec::from_iter(self.iter().map(|x| x.serialize())))
    }

    fn deserialize(expr: CstExpression) -> Result<Self, Error>
    where
        Self: Sized
    {
        let elements = deserialization_iter::<Self>(expr.clone())?;

        Ok(Result::from_iter(elements.map(|x| T::deserialize(x)))?)
    }
}
impl<K: Clone+Config+Hash+Eq, V: Clone+Config> Config for HashMap<K, V> {
    fn serialize(&self) -> CstExpression {
        CstExpression::collection(Vec::from_iter(
            self.iter().map(|(k, v)| (k.clone(), v.clone()).serialize())
        ))
    }

    fn deserialize(expr: CstExpression) -> Result<Self, Error> {
        let kv_pairs = deserialization_iter::<Self>(expr.clone())?;

        let mut hm = HashMap::new();
        for kv_pair in kv_pairs {
            let (k,v) = <(K, V)>::deserialize(kv_pair)?;
            hm.insert(k ,v);
        }
        Ok(hm)
    }
}
impl<T: Config> Config for Box<T> {
    fn serialize(&self) -> CstExpression {
        T::serialize(self)
    }

    fn deserialize(expr: CstExpression) -> Result<Self, Error> {
        Ok(Box::new(T::deserialize(expr)?))
    }
}

impl Config for () {
    fn serialize(&self) -> CstExpression {
        CstExpression::collection(vec![])
    }

    fn deserialize(expr: CstExpression) -> Result<Self, Error>
    where
        Self: Sized
    {
        match &expr.data {
            Presence(_) => Err(Error::at(ExpectedCollectionGot(expr.pretty()), expr.location)),
            CstData::Pair(_, _) => Err(Error::at(ExpectedCollectionGot(expr.pretty()), expr.location)),
            CstData::Collection(c) => {
                if !c.is_empty() {
                    Err(Error::at(WrongNumberOfElements(0, c.len()), expr.location))
                } else {
                    Ok(())
                }
            }
        }
    }
}


impl<T: Config> Config for Option<T> {
    fn serialize(&self) -> CstExpression {
        match self {
            None => CstExpression::presence(String::from("None")),
            Some(t) => CstExpression::collection(vec![CstExpression::presence(String::from("Some")), t.serialize()])
        }
    }

    fn deserialize(expr: CstExpression) -> Result<Self, Error>
    where
        Self: Sized
    {
        let mut fields = deserialization_iter::<Self>(expr.clone())?;

        let specifier = match &expr.data {
            Presence(s) => Some(s.clone()),
            Collection(c) => {
                let specifier = c.get(0).map(|x| x.release().map(|x| x.clone())).flatten();

                if specifier.is_some() {
                    fields.next();
                }

                specifier
            },
            _ => None
        }.ok_or(Error::at(ExpectedTypeGot(type_name::<T>().to_string(), expr.pretty()), expr.location))?;

        match specifier.as_str() {
            "Some" => Ok(Some(T::deserialize(fields.next().ok_or(Error::at(ExpectedTypeGot(type_name::<T>().to_string(), expr.pretty()), expr.location))?)?)),
            "None" => Ok(None),
            _ => Err(Error::at(ExpectedTypeGot(type_name::<T>().to_string(), expr.pretty()), expr.location))
        }
    }
}


