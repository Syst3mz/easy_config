use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;
use std::sync::Arc;
use crate::interner::{Interner, OptionExtension};
use crate::value::Value;

pub trait ToValue {
    fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value;
}

macro_rules! impl_to_value {
    ($typ:ty) => {
        impl ToValue for $typ {
            fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
                let text = string_interner.intern(self.to_string().as_str());
                Value::presence(text, comment.intern(string_interner))
            }
        }
    };
}

impl_to_value!(u8);
impl_to_value!(u16);
impl_to_value!(u32);
impl_to_value!(u64);
impl_to_value!(u128);
impl_to_value!(i8);
impl_to_value!(i16);
impl_to_value!(i32);
impl_to_value!(i64);
impl_to_value!(i128);
impl_to_value!(f32);
impl_to_value!(f64);
impl_to_value!(bool);
impl_to_value!(char);

impl ToValue for () {
    fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
        Value::list(None, [], comment.intern(string_interner))
    }
}
impl ToValue for String {
    fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
        // todo: Escape problem characters.
        let comment = comment.intern(string_interner);

        if self.is_empty() {
            return Value::list(None, [], comment);
        }

        let mut iter = self
            .split_ascii_whitespace()
            .map(|x| Value::presence(string_interner.intern(x), None))
            .peekable();

        let first = iter.next().unwrap();

        if iter.peek().is_none() {
            return first.with_comment(comment);
        }

        Value::list(None, std::iter::once(first).chain(iter), comment)
    }
}


macro_rules! impl_reference_type {
    ($typ:ty) => {
        impl<T: ToValue> ToValue for $typ {
            fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
                (**self).to_value(string_interner, comment)
            }
        }
    };
}
impl_reference_type!(&T);
impl_reference_type!(&mut T);
impl_reference_type!(Box<T>);
impl_reference_type!(Rc<T>);
impl_reference_type!(Arc<T>);

impl<T: ToValue> ToValue for Option<T> {
    fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
        match self {
            None => Value::presence(string_interner.intern("None"), comment.intern(string_interner)),
            Some(v) => Value::list(string_interner.intern("Some"), [v.to_value(string_interner, None)], comment.intern(string_interner))
        }
    }
}

impl<K: AsRef<str>, V: ToValue> ToValue for HashMap<K, V> {
    fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
        let comment = comment.intern(string_interner);
        Value::list(
            None,
            self.iter().map(|(k, v)| Value::binding(string_interner.intern(k), v.to_value(string_interner, None), None)),
            comment
        )
    }
}
impl<K: AsRef<str>, V: ToValue> ToValue for BTreeMap<K, V> {
    fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
        let comment = comment.intern(string_interner);
        Value::list(
            None,
            self.iter().map(|(k, v)| Value::binding(string_interner.intern(k), v.to_value(string_interner, None), None)),
            comment
        )
    }
}


impl<T: ToValue> ToValue for Vec<T> {
    fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
       self.as_slice().to_value(string_interner, comment)
    }
}

impl<T: ToValue> ToValue for [T] {
    fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
        let comment = comment.intern(string_interner);
        Value::list(None, self.iter().map(|x| x.to_value(string_interner, None)), comment)
    }
}

impl<T: ToValue, const N: usize> ToValue for [T; N] {
    fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
        self.as_slice().to_value(string_interner, comment)
    }
}

impl<T: ToValue> ToValue for std::collections::HashSet<T> {
    fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
        let comment = comment.intern(string_interner);
        Value::list(None, self.iter().map(|x| x.to_value(string_interner, None)), comment)
    }
}

impl<T: ToValue> ToValue for std::collections::BTreeSet<T> {
    fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
        let comment = comment.intern(string_interner);
        Value::list(None, self.iter().map(|x| x.to_value(string_interner, None)), comment)
    }
}

macro_rules! impl_to_value_tuple {
    ($($T:ident),+) => {
        impl<$($T: ToValue),+> ToValue for ($($T,)+) {
            fn to_value(&self, string_interner: &mut Interner, comment: Option<&'static str>) -> Value {
                let comment = comment.intern(string_interner);
                #[allow(non_snake_case)]
                let ($($T,)+) = self;
                Value::list(None, [$($T.to_value(string_interner, None)),+], comment)
            }
        }
    };
}

impl_to_value_tuple!(A);
impl_to_value_tuple!(A, B);
impl_to_value_tuple!(A, B, C);
impl_to_value_tuple!(A, B, C, D);
impl_to_value_tuple!(A, B, C, D, E);
impl_to_value_tuple!(A, B, C, D, E, F);
impl_to_value_tuple!(A, B, C, D, E, F, G);
impl_to_value_tuple!(A, B, C, D, E, F, G, H);
impl_to_value_tuple!(A, B, C, D, E, F, G, H, I);
impl_to_value_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_to_value_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_to_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_to_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_to_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_to_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_to_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_to_value_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);