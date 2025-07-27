use crate::config_error::Contextualize;
use crate::expression::{Atom, Expression, ExpressionData};
use crate::expression_iterator::ExpressionIterator;
use crate::lexer;
use crate::serialization::{Config};
use crate::serialization::Kind;
use crate::serialization::option_span_combine::OptionSpanCombine;
use crate::serialization::serialization_error::SerializationError;

macro_rules! config {
    ($ty: ty) => {
        impl Config for $ty {
            fn serialize(&self) -> Expression {
                Expression::presence(*self)
            }

            fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError> {
                let source_text = source_text.as_ref();
                let expr = exprs.next_or_err(source_text)?;
                let span = expr.span();
                match expr.data {
                    ExpressionData::Presence(p, _) => match p {
                        Atom::Number(n) => Ok(n.parse()?),
                        _ => Err(SerializationError::on_span(Kind::ExpectedNumber(p.to_string()), span, source_text))
                        .contextualize(format!("Error while deserializing a {}", stringify!($ty))),
                    },
                    _ => Err(SerializationError::on_span(Kind::ExpectedPresence(expr), span, source_text))
                    .contextualize(format!("Error while deserializing a {}", stringify!($ty)))
                }
            }
        }
    };

    ($ty: ty, non_numeric) => {
        impl Config for $ty {
            fn serialize(&self) -> Expression {
                Expression::presence(self.to_string())
            }

            fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError> {
                let source_text = source_text.as_ref();
                let expr = exprs.next_or_err(source_text)?;
                let span = expr.span();
                match expr.data {
                    ExpressionData::Presence(p, _) => match p {
                        Atom::Text(t) => Ok(t.parse()?),
                        _ => Err(SerializationError::on_span(Kind::ExpectedText(p.to_string()), span, source_text))
                        .contextualize(format!("Error while deserializing a {}", stringify!($ty))),
                    },
                    _ => Err(SerializationError::on_span(Kind::ExpectedPresence(expr), span, source_text))
                    .contextualize(format!("Error while deserializing a {}", stringify!($ty)))
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

fn escape_string(text: impl AsRef<str>) -> String {
    let mut text = text.as_ref().to_string();
    for stopping_char in lexer::STOPPING_CHARS {
        text = text.replace(&format!("{}", stopping_char), &format!("\\{}", stopping_char))
    }

    text.to_string()
}
impl Config for String {
    fn serialize(&self) -> Expression {
        Expression::list(
            self.split(" ")
                .map(|x| Expression::presence(escape_string(x)))
                .collect::<Vec<Expression>>(),
        ).minimized()
    }

    fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let source_text = source_text.as_ref();
        let mut span = None;

        for expr in exprs {
            let ExpressionData::Presence(_, s) = &expr.data else {
                return Err(SerializationError::on_span(Kind::ExpectedPresence(expr), span.unwrap(), source_text))
                    .contextualize("Error while deserializing a String");
            };

            span.combine(*s)
        }

        let span = span.unwrap();
        Ok(source_text[span.start()..span.end()].to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::Parser;
    use super::*;

    #[test]
    fn serialize_string() {
        let expected = "hi".to_string();
        assert_eq!(expected.serialize(), Expression::presence("hi"));
    }
    #[test]
    fn serialize_string_with_space() {
        let expected = "hi there".to_string();
        assert_eq!(expected.serialize(), Expression::list(
    vec![
            Expression::presence("hi"),
            Expression::presence("there")
        ]))
    }

    #[test]
    fn deserialize_string() {
        let expected = "hi".to_string();
        let got = String::deserialize(&mut Parser::new(&expected).parse().unwrap().into_iter(), &expected).unwrap();
        assert_eq!(got, expected)
    }

    #[test]
    fn deserialize_string_with_space() {
        let expected = "(hi there)".to_string();
        let parsed = Parser::new(&expected).parse().unwrap();
        let mut parsed = parsed.into_iter();
        let got = String::deserialize(&mut parsed.next().unwrap().into_iter(), &expected).unwrap();
        assert_eq!(got, "hi there")
    }
}