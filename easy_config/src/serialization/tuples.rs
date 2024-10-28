use std::any::type_name;
use crate::serialization::{Config, DeserializeExtension};
use crate::serialization::error::Error;
use crate::serialization::CstExpression;
use crate::serialization::error::Kind::ExpectedCollectionGot;
use crate::serialization::error::Kind::ExpectedTypeGot;

macro_rules! impl_tuple {
    ($($typ: ident),*) => {
        impl< $( $typ: Config ),* > Config for ( $($typ),* ) {
            #[allow(non_snake_case)]
            fn serialize(&self) -> CstExpression {
                let ($(ref $typ),*) = *self;  // Destructure the tuple

                // Collect serialized elements into a vector
                CstExpression::collection(vec![
                    $(
                        $typ.serialize()  // Call serialize on each element
                    ),*
                ])
            }


            fn deserialize(expr: CstExpression) -> Result<Self, Error>
            where
                Self: Sized
            {
                let pretty = expr.pretty();
                let location = expr.location;


                let mut x = expr.into_deserialization_iterator()
                    .ok_or(Error::at(ExpectedCollectionGot(pretty.clone()), location))?;
                Ok((
                    $(
                        $typ::deserialize(x.next()
                            .ok_or(Error::at(ExpectedTypeGot(type_name::<$typ>().to_string(), pretty.clone()), location))?)?
                    ),*
                ))
            }
        }
    };
}

impl_tuple!(T1, T2);
impl_tuple!(T1, T2, T3);
impl_tuple!(T1, T2, T3, T4);
impl_tuple!(T1, T2, T3, T4, T5);
impl_tuple!(T1, T2, T3, T4, T5, T6);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);