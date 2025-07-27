use std::any;
use std::collections::HashMap;
use crate::config_error::Contextualize;
use crate::expression::{Expression, ExpressionData};
use crate::expression_iterator::ExpressionIterator;
use crate::lexical_span::LexicalSpan;
use crate::serialization::Config;
use crate::serialization::serialization_error::{Kind, SerializationError};

fn serialize_linear<'a, T: Config>(elements: impl Iterator<Item=&'a T>) -> Expression {
    Expression::list(elements.map(|x| x.serialize()).collect()).minimized()
}

fn deserialize_linear<'a, T: Config>(elements: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Vec<T>, SerializationError> {
    let source_text = source_text.as_ref();

    if T::PASSTHROUGH {
        let mut collection = Vec::new();
        while elements.len() != 0 {
            collection.push(T::deserialize(elements, source_text)?)
        };

        return Ok(collection)
    }

    Result::from_iter(elements.map(|x| {
        T::deserialize(&mut x.into_iter(), source_text)
    }))
}

impl<T: Config> Config for Vec<T> {
    fn serialize(&self) -> Expression {
        serialize_linear(self.iter())
    }

    fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError> {
        let source_text = source_text.as_ref();
        deserialize_linear(exprs, source_text)
            .contextualize(format!("Error while deserializing Vec<{}>", any::type_name::<T>()))
    }
}
impl<T: Config, const N: usize> Config for [T; N] {
    fn serialize(&self) -> Expression {
        serialize_linear(self.iter())
    }

    fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let source_text = source_text.as_ref();
        let deserialized = deserialize_linear(exprs, source_text)
            .contextualize(format!("Error while deserializing [{}; {}]", any::type_name::<T>(), N))?;
        let deserialized_len = deserialized.len();
        deserialized.try_into().map_err(|_| SerializationError::on_span(
            Kind::WrongCardinality {
                got: deserialized_len,
                want: N,
            },
            exprs
                .span()
                .unwrap_or(LexicalSpan::zeros()),
            source_text
        ).contextualize(format!("Error while deserializing [{}; {}]", any::type_name::<T>(), N)))
    }
}

fn deserialize_hashmap_binding<T: Config>(expression: Expression, source_text: impl AsRef<str>) -> Result<(String, T), SerializationError> {
    let span = expression.span();
    let ExpressionData::BindingExpr(binding) = expression.data else {
        return Err(SerializationError::on_span(
            Kind::ExpectedBinding(expression),
            span,
            source_text
        ))
    };

    Ok((binding.name, T::deserialize(&mut binding.value.into_iter(), source_text)?))
}
impl<T: Config> Config for HashMap<String, T> {
    fn serialize(&self) -> Expression {
        Expression::list(
            self.iter()
                .map(|(k, v)| Expression::binding(
                    k.to_string(),
                    v.serialize(),
                ))
                .collect(),
        ).minimized()
    }
    fn deserialize(exprs: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError> {
        let source_text = source_text.as_ref();
        let list: Vec<(String, T)> = Result::from_iter(exprs.map(|x| deserialize_hashmap_binding(x, source_text)))
            .contextualize(format!("Error while deserializing Hashmap<String, {}>", any::type_name::<T>()))?;
        Ok(HashMap::from_iter(list))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_serialize() {
        let expected = vec![1, 2, 3];
        assert_eq!(expected.serialize(), Expression::list(vec![
            Expression::presence(1),
            Expression::presence(2),
            Expression::presence(3),
        ]))
    }
    #[test]
    fn vec_deserialize() {
        let expected = vec![1, 2, 3];
        let got = Vec::<i32>::deserialize(&mut expected.serialize().into_iter(),"(1 2 3)").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn array() {
        let expected = [1, 2, 3];
        let got = <[i32; 3]>::deserialize(&mut expected.serialize().into_iter(),"(1 2 3)").unwrap();
        assert_eq!(expected, got);
    }

    #[test]
    fn hashmap_string() {
        let expected = HashMap::from([
            (String::from("a"), 1),
            (String::from("b"), 2),
            (String::from("c"), 3),
        ]);
        let got = HashMap::<String, i32>::deserialize(&mut expected.serialize().into_iter(),"(a=1 b=2 c=3)").unwrap();
        if got.len() != expected.len() {
            panic!("length mismatch: {:?} != {:?}", got, expected);
        }

        for (k, v) in got {
            assert_eq!(Some(&v), expected.get(&k));
        }
    }
}