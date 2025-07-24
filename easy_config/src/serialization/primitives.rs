use crate::expression::{Atom, Expression, ExpressionData};
use crate::lexical_range::LexicalSpan;
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
