use crate::expression::{Atom, Expression, ExpressionData};
use crate::lexical_span::LexicalSpan;
use crate::serialization::{Config};
use crate::serialization::Kind;
use crate::serialization::serialization_error::SerializationError;

macro_rules! config {
    ($ty: ty) => {
        impl Config for $ty {
            fn serialize(&self) -> Expression {
                Expression::presence(*self, LexicalSpan::zeros())
            }

            fn deserialize(expr: Expression, source_text: impl AsRef<str>) -> Result<Self, SerializationError> {
                let span = expr.span();
                match expr.data {
                    ExpressionData::Presence(p, _) => match p {
                        Atom::Number(n) => Ok(n.parse()?),
                        _ => Err(SerializationError::on_span(Kind::ExpectedNumber(p.to_string()), span, source_text)),
                    },
                    _ => Err(SerializationError::on_span(Kind::ExpectedPresence(expr), span, source_text))
                }
            }
        }
    };

    ($ty: ty, non_numeric) => {
        impl Config for $ty {
            fn serialize(&self) -> Expression {
                Expression::presence(self.to_string(), LexicalSpan::zeros())
            }

            fn deserialize(expr: Expression, source_text: impl AsRef<str>) -> Result<Self, SerializationError> {
                let span = expr.span();
                match expr.data {
                    ExpressionData::Presence(p, _) => match p {
                        Atom::Text(t) => Ok(t.parse()?),
                        _ => Err(SerializationError::on_span(Kind::ExpectedText(p.to_string()), span, source_text)),
                    },
                    _ => Err(SerializationError::on_span(Kind::ExpectedPresence(expr), span, source_text))
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

config!(bool, non_numeric);
config!(char, non_numeric);

impl Config for String {
    fn serialize(&self) -> Expression {
        Expression::list(
            self.split(" ")
                .map(|x| Expression::presence(x, LexicalSpan::zeros()))
                .collect::<Vec<Expression>>(),
            LexicalSpan::zeros()
        ).minimized()
    }

    fn deserialize(expr: Expression, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let source_text = source_text.as_ref();

        let mut spans = expr.into_iter().map(|x| {
            let span = x.span();
            match x.data {
                ExpressionData::Presence(_, span) => Ok(span),
                _ => Err(SerializationError::on_span(Kind::ExpectedPresence(x), span, source_text))
            }
        });

        let mut bounding_span = spans.next().unwrap()?;
        for span in spans {
            let Ok(span) = span else { continue };
            bounding_span = bounding_span.combine(span);
        }

        Ok(bounding_span.slice(source_text).to_string())
    }
}