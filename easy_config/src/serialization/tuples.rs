use crate::expression_iterator::ExpressionIterator;
use crate::lexical_span::LexicalSpan;
use crate::serialization::{Config};
use crate::serialization::Expression;
use crate::serialization::serialization_error::{Kind, SerializationError};
use crate::serialization::option_span_combine::OptionSpanCombine;


macro_rules! impl_tuple {
    ($($typ:ident),*) => {
        impl<$( $typ: Config ),*> Config for ($($typ,)*) {
            #[allow(non_snake_case)]
            fn serialize(&self) -> Expression {
                let ($(ref $typ),*) = *self;

                Expression::list(vec![
                    $(
                        $typ.serialize()
                    ),*
                ])
            }

            fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
            where
                Self: Sized
            {
                let source_text = source_text.as_ref();

                let cardnality = <[()]>::len(&[$(impl_tuple!(@sub $typ)),*]);
                let mut span = None;
                let mut count = 0;
                Ok((
                    $(
                        $typ::deserialize(
                            &mut exprs
                                .next()
                                .map(|x| {span.combine(x.span()); count += 1; x})
                                .ok_or(
                                    SerializationError::on_span(
                                        Kind::WrongCardinality {
                                            want: cardnality,
                                            got: count,
                                        },
                                        span.unwrap_or(LexicalSpan::zeros()),
                                        source_text
                                ))?
                                .into_iter(),
                            source_text
                        )?
                    ),*
                ))
            }
        }
    };

    (@sub $t:ident) => { () };
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
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);
impl_tuple!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_ply() {
        let start = (1, 2);
        assert_eq!(<(i32, i32)>::deserialize(&mut start.serialize().into_iter(), "(1 2)").unwrap(), (1, 2));
    }

    #[test]
    fn two_ply_distinct_types() {
        let start = (1, 3_u32);
        assert_eq!(<(i32, u32)>::deserialize(&mut start.serialize().into_iter(), "(1 3)").unwrap(), (1, 3_u32));
    }
}