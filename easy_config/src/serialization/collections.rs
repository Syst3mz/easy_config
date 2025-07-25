use std::collections::HashMap;
use crate::expression::{Expression, ExpressionData};
use crate::lexical_span::LexicalSpan;
use crate::serialization::Config;
use crate::serialization::serialization_error::{Kind, SerializationError};

impl<T: Config> Config for Vec<T> {
    fn serialize(&self) -> Expression {
        Expression::list(self.iter().map(|x| x.serialize()).collect(), LexicalSpan::zeros()).minimized()
    }

    fn deserialize(expr: Expression, source_text: impl AsRef<str>) -> Result<Self, SerializationError> {
        let source_text = source_text.as_ref();
        Result::from_iter(expr.into_iter().map(|x| T::deserialize(x, source_text)))
    }
}
impl<T: Config, const N: usize> Config for [T; N] {
    fn serialize(&self) -> Expression {
        Expression::list(self.iter().map(|x| x.serialize()).collect(), LexicalSpan::zeros()).minimized()
    }

    fn deserialize(expr: Expression, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let span = expr.span();
        let source_text = source_text.as_ref();
        let list: Vec<T> = Result::from_iter(expr.into_iter().map(|x| T::deserialize(x, source_text)))?;

        let list_len = list.len();
        match list.try_into() {
            Ok(x) => Ok(x),
            Err(_) => Err(SerializationError::on_span(Kind::WrongCardinality {
                got: list_len,
                want: N,
            }, span, source_text))
        }
    }
}

fn deserialize_hashmap_binding<T: Config>(expression: Expression, source_text: impl AsRef<str>) -> Result<(String, T), SerializationError> {
    let span = expression.span();
    let ExpressionData::Binding(key, v, _) = expression.data else {
        return Err(SerializationError::on_span(
            Kind::ExpectedBinding(expression),
            span,
            source_text
        ))
    };

    Ok((key, T::deserialize(*v, source_text)?))
}
impl<T: Config> Config for HashMap<String, T> {
    fn serialize(&self) -> Expression {
        Expression::list(
            self.iter()
                .map(|(k, v)| Expression::binding(
                    k.to_string(),
                    v.serialize(),
                    LexicalSpan::zeros()
                ))
                .collect(),
            LexicalSpan::zeros()
        ).minimized()
    }
    fn deserialize(expr: Expression, source_text: impl AsRef<str>) -> Result<Self, SerializationError> {
        let source_text = source_text.as_ref();
        let list: Vec<(String, T)> = Result::from_iter(expr.into_iter().map(|x| deserialize_hashmap_binding(x, source_text)))?;
        Ok(HashMap::from_iter(list))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec() {
        let expected = vec![1, 2, 3];
        let got = Vec::<i32>::deserialize(expected.serialize(),"(1 2 3)").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn array() {
        let expected = [1, 2, 3];
        let got = <[i32; 3]>::deserialize(expected.serialize(),"(1 2 3)").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn hashmap_string() {
        let expected = HashMap::from([
            (String::from("a"), 1),
            (String::from("b"), 2),
            (String::from("c"), 3),
        ]);
        let got = HashMap::<String, i32>::deserialize(expected.serialize(),"(a=1 b=2 c=3)").unwrap();
        if got.len() != expected.len() {
            panic!("length mismatch: {:?} != {:?}", got, expected);
        }

        for (k, v) in got {
            assert_eq!(Some(&v), expected.get(&k));
        }
    }
}