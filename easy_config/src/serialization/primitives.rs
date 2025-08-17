use crate::config_error::Contextualize;
use crate::expression::{Atom, Expression, ExpressionData};
use crate::expression_iterator::ExpressionIterator;
use crate::lexer;
use crate::serialization::{EasyConfig};
use crate::serialization::Kind;
use crate::serialization::option_span_combine::OptionSpanCombine;
use crate::serialization::serialization_error::SerializationError;

macro_rules! config {
    ($ty: ty) => {
        impl EasyConfig for $ty {
            fn serialize(&self) -> Expression {
                Expression::presence(*self)
            }

            fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError> {
                let source_text = source_text.as_ref();
                let expr = exprs.minimized_next_or_err(source_text)?;
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
        impl EasyConfig for $ty {
            fn serialize(&self) -> Expression {
                Expression::presence(self.to_string())
            }

            fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError> {
                let source_text = source_text.as_ref();
                let expr = exprs.minimized_next_or_err(source_text)?;
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
    let mut escaped = String::with_capacity(text.as_ref().len());
    for ch in text.as_ref().chars() {
        if lexer::STOPPING_CHARS.contains(&ch) {
            escaped.push('\\');
        }
        escaped.push(ch);
    }
    escaped
}


fn deserialize_string(exprs: &mut ExpressionIterator, source_text: &str) -> Result<String, SerializationError> {
    let mut span = None;
    for expr in exprs {
        span.combine(expr.span());

        let ExpressionData::Presence(_, _) = &expr.data else {
            return Err(SerializationError::on_span(Kind::ExpectedPresence(expr), span.unwrap(), source_text))
                .contextualize("Error while deserializing a String");
        };
    }

    let span = span.unwrap();
    Ok(source_text[span.start()..span.end()].to_string())
}
impl EasyConfig for String {
    fn serialize(&self) -> Expression {
        Expression::list(
            self.split_whitespace()
                .map(|x| Expression::presence(escape_string(x)))
                .collect::<Vec<Expression>>(),
        ).minimized()
    }

    fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let source_text = source_text.as_ref();

        if exprs.finished() {
            return Err(SerializationError::end_of_input(source_text))    
        }

        let peeked = exprs.peek().unwrap();
        if peeked.is_list() {
            deserialize_string(&mut exprs.next().unwrap().into_iter(), source_text)
        } else {
            deserialize_string(exprs, source_text)
        }
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

    #[test]
    fn deserialize_bound_string_with_space() {
        let content = "hi there";
        let source = format!("x = ({})", content);
        let parsed = Parser::new(&source).parse().unwrap();
        let mut parsed = parsed.into_iter();
        let ExpressionData::BindingExpr(b) = parsed.next().unwrap().data else {panic!("Expected binding")};
        if b.name != "x" {
            panic!("expected x but got {}", b.name)
        }
        let got = String::deserialize(&mut b.value.into_iter(), &source).unwrap();
        assert_eq!(got, content)
    }
}