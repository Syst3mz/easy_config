use crate::expression::ExpressionData;
use crate::lexical_span::LexicalSpan;
use crate::serialization::{Config};
use crate::serialization::Expression;
use crate::serialization::serialization_error::{Kind, SerializationError};

trait Cardinality {
    const CARDINALITY: usize;
}

fn extract_expr(exprs: &mut impl Iterator<Item=(usize, Expression)>, last: &mut usize, cardinality: usize, span: LexicalSpan, source_text: impl AsRef<str>) -> Result<Expression, SerializationError> {
    exprs
        .next()
        .ok_or(SerializationError::on_span(Kind::WrongCardinality {
            got: *last + 1,
            want: cardinality,
        }, span, source_text))
        .map(|(index, expr)| {
            *last = index;
            expr
        })
}

macro_rules! impl_tuple {
    ($($typ:ident),*) => {
        impl<$($typ),*> Cardinality for ($($typ,)*) {
            const CARDINALITY: usize = <[()]>::len(&[$(impl_tuple!(@sub $typ)),*]);
        }

        impl<$( $typ: Config ),*> Config for ($($typ,)*) {
            #[allow(non_snake_case)]
            fn serialize(&self) -> Expression {
                let ($(ref $typ),*) = *self;

                Expression::list(vec![
                    $(
                        $typ.serialize()
                    ),*
                ], LexicalSpan::zeros())
            }

            fn deserialize(expr: Expression, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
            where
                Self: Sized
            {
                let source_text = source_text.as_ref();
                let span = expr.span();
                let ExpressionData::List(list, _) = expr.data else {
                    return Err(SerializationError::on_span(Kind::ExpectedList(expr), span, source_text))
                };
                let mut list_elements = list.into_iter().enumerate();
                let mut counter = 0;

                Ok((
                    $(
                        $typ::deserialize(
                            extract_expr(&mut list_elements, &mut counter, Self::CARDINALITY, span, source_text)?,
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
        assert_eq!(<(i32, i32)>::deserialize(start.serialize(), "(1 2)").unwrap(), (1, 2));
    }

    #[test]
    fn two_ply_distinct_types() {
        let start = (1, 3_u32);
        assert_eq!(<(i32, u32)>::deserialize(start.serialize(), "(1 3)").unwrap(), (1, 3_u32));
    }
}