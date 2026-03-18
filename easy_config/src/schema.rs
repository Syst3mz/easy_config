use std::collections::{BTreeMap, HashMap};

type Static<T> = &'static T;

pub trait HasSchema {
    const SCHEMA: Schema;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamedField {
    pub name: Static<str>,
    pub schema: Schema,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fields {
    Unit,
    NewType(Static<Schema>),
    Tuple(Static<[Schema]>),
    Named(Static<[NamedField]>),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NamedBagOfFields {
    pub name: Static<str>,
    pub fields: Fields,
}

pub type StructSchema = NamedBagOfFields;
pub type Variant = NamedBagOfFields;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnumSchema {
    pub name: Static<str>,
    pub variants: Static<[Variant]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Schema {
    Bool,
    I8, I16, I32, I64,
    U8, U16, U32, U64,
    F32, F64,
    Char,
    String,
    ByteArray,
    Option(Static<Schema>),
    Unit,
    Sequence(Static<Schema>),
    Tuple(Static<[Schema]>),
    Map(Static<Schema>, Static<Schema>),
    Struct(Static<StructSchema>),
    Enum(Static<EnumSchema>),
}

macro_rules! parameterless_schema {
    ($typ:ty, $schema:ident) => {
        impl HasSchema for $typ {
            const SCHEMA: Schema = Schema::$schema;
        }
    };
}

parameterless_schema!(bool, Bool);
parameterless_schema!(i8, I8);
parameterless_schema!(i16, I16);
parameterless_schema!(i32, I32);
parameterless_schema!(i64, I64);
parameterless_schema!(u8, U8);
parameterless_schema!(u16, U16);
parameterless_schema!(u32, U32);
parameterless_schema!(u64, U64);
parameterless_schema!(f32, F32);
parameterless_schema!(f64, F64);
parameterless_schema!(char, Char);
parameterless_schema!(String, String);
parameterless_schema!((), Unit);

impl<T: HasSchema> HasSchema for Option<T> {
    const SCHEMA: Schema = Schema::Option(&T::SCHEMA);
}

impl<K: HasSchema, V: HasSchema> HasSchema for HashMap<K, V> {
    const SCHEMA: Schema = Schema::Map(&K::SCHEMA, &V::SCHEMA);
}
impl<K: HasSchema, V: HasSchema> HasSchema for BTreeMap<K, V> {
    const SCHEMA: Schema = Schema::Map(&K::SCHEMA, &V::SCHEMA);
}

impl<T: HasSchema> HasSchema for &T {
    const SCHEMA: Schema = T::SCHEMA;
}
impl<T: HasSchema> HasSchema for &mut T {
    const SCHEMA: Schema = T::SCHEMA;
}
impl<T: HasSchema> HasSchema for Box<T> {
    const SCHEMA: Schema = T::SCHEMA;
}
impl<T: HasSchema> HasSchema for std::rc::Rc<T> {
    const SCHEMA: Schema = T::SCHEMA;
}
impl<T: HasSchema> HasSchema for std::sync::Arc<T> {
    const SCHEMA: Schema = T::SCHEMA;
}
impl<T: HasSchema> HasSchema for Vec<T> {
    const SCHEMA: Schema = Schema::Sequence(&T::SCHEMA);
}

impl<T: HasSchema> HasSchema for [T] {
    const SCHEMA: Schema = Schema::Sequence(&T::SCHEMA);
}
impl<T: HasSchema, const N: usize> HasSchema for [T; N] {
    const SCHEMA: Schema = Schema::Sequence(&T::SCHEMA);
}
impl<T: HasSchema> HasSchema for std::collections::HashSet<T> {
    const SCHEMA: Schema = Schema::Sequence(&T::SCHEMA);
}
impl<T: HasSchema> HasSchema for std::collections::BTreeSet<T> {
    const SCHEMA: Schema = Schema::Sequence(&T::SCHEMA);
}

macro_rules! impl_tuple {
    ($($T:ident),+) => {
        impl<$($T: HasSchema),+> HasSchema for ($($T,)+) {
            const SCHEMA: Schema = Schema::Tuple(&[$($T::SCHEMA),+]);
        }
    };
}

impl_tuple!(A);
impl_tuple!(A, B);
impl_tuple!(A, B, C);
impl_tuple!(A, B, C, D);
impl_tuple!(A, B, C, D, E);
impl_tuple!(A, B, C, D, E, F);
impl_tuple!(A, B, C, D, E, F, G);
impl_tuple!(A, B, C, D, E, F, G, H);
impl_tuple!(A, B, C, D, E, F, G, H, I);
impl_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
impl_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
