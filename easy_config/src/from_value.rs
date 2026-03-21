use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::Arc;
use Error::FailedToParseValueAt;
use crate::error::Error;
use crate::span::Span;
use crate::value::{Value, ValueKind, BINDING_STR, PRESENCE_STR};

pub trait FromValue {
    fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error> where Self: Sized;
}

macro_rules! impl_from_value_parse {
    ($typ:ty) => {
        impl FromValue for $typ {
            fn from_value(value: Value, _: impl AsRef<str>) -> Result<Self, Error> {
                let span = value.span.unwrap_or(Span::new(0, 0));
                let value = value.to_presence()?;

                let text = value.resolve();
                text.parse::<$typ>().map_err(|_| FailedToParseValueAt {
                    expected: stringify!($typ),
                    got: text.to_string(),
                    span
                })
            }
        }
    };
}

impl_from_value_parse!(bool);
impl_from_value_parse!(u8);
impl_from_value_parse!(u16);
impl_from_value_parse!(u32);
impl_from_value_parse!(u64);
impl_from_value_parse!(i8);
impl_from_value_parse!(i16);
impl_from_value_parse!(i32);
impl_from_value_parse!(i64);
impl_from_value_parse!(f32);
impl_from_value_parse!(f64);
impl_from_value_parse!(char);

impl FromValue for String {
    fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error>
    where
        Self: Sized
    {
        let span = value.span.unwrap_or(Span::new(0, 0));
        let (_, values) = match value.kind {
            ValueKind::Presence(p) => return Ok(p.resolve().to_string()),
            ValueKind::Binding(_, _) => return Err(Error::WrongShape {
                expected: &[PRESENCE_STR, BINDING_STR],
                got: BINDING_STR,
                span,
            }),
            ValueKind::List(n, v) => (n, v)
        };

        let mut values = values
            .into_iter()
            .map(|x| {
                let span = x.span.unwrap_or(Span::new(0, 0));
                x.to_presence().map(|y| (y, span))
            });
        let Some(first) = values.next() else { return Ok(String::new()) };
        let Some(last) = values.last() else { return Ok(first?.0.resolve().to_string()) };
        let (_, first_span) = first?;
        let (_, last_span) = last?;

        Ok(source.as_ref()[first_span.merge(last_span).to_range()].to_string())
    }
}

impl<T: FromValue> FromValue for Box<T> {
    fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error> {
        Ok(Box::new(T::from_value(value, source)?))
    }
}

impl<T: FromValue> FromValue for Rc<T> {
    fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error> {
        Ok(Rc::new(T::from_value(value, source)?))
    }
}

impl<T: FromValue> FromValue for Arc<T> {
    fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error> {
        Ok(Arc::new(T::from_value(value, source)?))
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error> {
        if value.is_none_equivalent() {
            return Ok(None);
        }

        // Handle Some(value) — a list named "Some" with one element
        if let ValueKind::List(ref name, _) = value.kind {
            if name.as_ref().map(|n| n.resolve() == "Some").unwrap_or(false) {
                let (_, mut values) = value.to_list()?;
                let inner = values.remove(0);
                return Ok(Some(T::from_value(inner, source)?));
            }
        }

        // Bare value — stretch goal implicit Some
        Ok(Some(T::from_value(value, source)?))
    }
}

impl<T: FromValue> FromValue for Vec<T> {
    fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error> where Self: Sized {
        let (_, values) = value.to_list()?;
        values.into_iter()
            .map(|v| T::from_value(v, source.as_ref()))
            .collect()
    }
}

impl<T: FromValue, const N: usize> FromValue for [T; N] {
    fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error> where Self: Sized {
        let span = value.span.unwrap_or(Span::new(0, 0));
        let (_, values) = value.to_list()?;
        if values.len() != N {
            return Err(Error::WrongLength {
                expected: N,
                got: values.len(),
                span,
            });
        }
        let iter = values.into_iter().map(|v| T::from_value(v, source.as_ref()));
        let collected = iter.collect::<Result<Vec<T>, _>>()?;
        Ok(collected.try_into().unwrap_or_else(|_| unreachable!()))
    }
}

impl<T: FromValue + Eq + std::hash::Hash> FromValue for std::collections::HashSet<T> {
    fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error> where Self: Sized {
        let (_, values) = value.to_list()?;
        values.into_iter()
            .map(|v| T::from_value(v, source.as_ref()))
            .collect()
    }
}

impl<T: FromValue + Ord> FromValue for std::collections::BTreeSet<T> {
    fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error> where Self: Sized {
        let (_, values) = value.to_list()?;
        values.into_iter()
            .map(|v| T::from_value(v, source.as_ref()))
            .collect()
    }
}

impl<K: FromValue + Eq + std::hash::Hash, V: FromValue> FromValue for HashMap<K, V> {
    fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error> where Self: Sized {
        let (_, values) = value.to_list()?;
        values.into_iter()
            .map(|v| {
                let (k, v) = v.to_binding()?;
                let k = K::from_value(Value::presence(k, None), source.as_ref())?;
                let v = V::from_value(v, source.as_ref())?;
                Ok((k, v))
            })
            .collect()
    }
}

impl<K: FromValue + Ord, V: FromValue> FromValue for BTreeMap<K, V> {
    fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error> where Self: Sized {
        let (_, values) = value.to_list()?;
        values.into_iter()
            .map(|v| {
                let (k, v) = v.to_binding()?;
                let k = K::from_value(Value::presence(k, None), source.as_ref())?;
                let v = V::from_value(v, source.as_ref())?;
                Ok((k, v))
            })
            .collect()
    }
}

macro_rules! impl_from_value_tuple {
    ($($T:ident),+) => {
        impl<$($T: FromValue),+> FromValue for ($($T,)+) {
            fn from_value(value: Value, source: impl AsRef<str>) -> Result<Self, Error> where Self: Sized {
                let span = value.span.unwrap_or(Span::new(0, 0));
                let (_, values) = value.to_list()?;
                let expected = [$(stringify!($T)),+].len();
                if values.len() != expected {
                    return Err(Error::WrongLength {
                        expected,
                        got: values.len(),
                        span,
                    });
                }
                let mut iter = values.into_iter();
                Ok(($($T::from_value(iter.next().unwrap(), source.as_ref())?,)+))
            }
        }
    };
}

impl FromValue for () {
    fn from_value(value: Value, _: impl AsRef<str>) -> Result<Self, Error> {
        let span = value.span.unwrap_or(Span::new(0, 0));
        let (_, values) = value.to_list()?;
        if !values.is_empty() {
            return Err(Error::WrongLength {
                expected: 0,
                got: values.len(),
                span,
            });
        }
        Ok(())
    }
}

impl_from_value_tuple!(A);
impl_from_value_tuple!(A, B);
impl_from_value_tuple!(A, B, C);
impl_from_value_tuple!(A, B, C, D);
impl_from_value_tuple!(A, B, C, D, E);
impl_from_value_tuple!(A, B, C, D, E, F);
impl_from_value_tuple!(A, B, C, D, E, F, G);
impl_from_value_tuple!(A, B, C, D, E, F, G, H);
impl_from_value_tuple!(A, B, C, D, E, F, G, H, I);
impl_from_value_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_from_value_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_from_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_from_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_from_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_from_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_from_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_from_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);