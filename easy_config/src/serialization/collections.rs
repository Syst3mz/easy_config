use crate::config_error::Contextualize;
use crate::expression::ExpressionData::List;
use crate::expression::{Expression, ExpressionData};
use crate::expression_iterator::ExpressionIterator;
use crate::lexical_span::LexicalSpan;
use crate::serialization::serialization_error::Kind::{ExpectedBinding, ExpectedList, WrongCardinality};
use crate::serialization::serialization_error::SerializationError;
use crate::serialization::EasyConfig;
use std::collections::HashMap;
fn serialize_array_like<'a, T: EasyConfig>(iter: impl Iterator<Item=&'a T>) -> Expression {
    Expression::list(iter.map(|e| e.serialize()).collect())
}
fn deserialize_array_like<T: EasyConfig>(expression_iterator: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<(Vec<T>, LexicalSpan), SerializationError> {
    let source_text = source_text.as_ref();
    let next = expression_iterator.next_or_err()?;

    let List(exprs, span) = next.data else {
        let span = next.span();
        return Err(SerializationError::on_span(ExpectedList(next), span));
    };

    if exprs.is_empty() {
        return Ok((vec![], span));
    }

    if !T::is_composite() {
        let mut results = Vec::new();
        for expr in exprs {
            let wrapper = Expression::list(vec![expr]);
            let mut iter = wrapper.into_iter();
            results.push(T::deserialize(&mut iter, source_text)?);
        }
        return Ok((results, span));
    }

    let mut ret = vec![];
    let mut iter = Expression::new(List(exprs, span), next.comment).into_iter();
    loop {
        if iter.finished() {
            break;
        }
        ret.push(T::deserialize(&mut iter, source_text)?);
    }

    Ok((ret, span))
}

impl<T: EasyConfig> EasyConfig for Vec<T> {

    fn serialize(&self) -> Expression {
        serialize_array_like(self.iter())
    }

    fn deserialize(expression_iterator: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let deserialized_vec = deserialize_array_like(expression_iterator, source_text)
            .contextualize("Failed to deserialize vector.")?;
        Ok(deserialized_vec.0)
    }
}
impl<T: EasyConfig, const N: usize> EasyConfig for [T; N] {
    fn serialize(&self) -> Expression {
        serialize_array_like(self.iter())
    }

    fn deserialize(expression_iterator: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let (deserialized_vec, span) = deserialize_array_like::<T>(expression_iterator, source_text)
            .contextualize("Failed to deserialize vector.")?;

        let err = SerializationError::on_span(WrongCardinality { got: deserialized_vec.len(), want: N }, span);
        if deserialized_vec.len() != N {
            return Err(err);
        }

        deserialized_vec.try_into().map_err(|_| err)
    }
}
impl<T: EasyConfig> EasyConfig for HashMap<String, T> {
    fn serialize(&self) -> Expression {
        Expression::list(self.iter().map(|(k, v)| Expression::binding(k, v.serialize())).collect())
    }

    fn deserialize(expression_iterator: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let source_text = source_text.as_ref();
        let next = expression_iterator.next_or_err()?;

        let List(exprs, _) = next.data else {
            let span = next.span();
            return Err(SerializationError::on_span(ExpectedList(next), span));
        };

        let mut ret = HashMap::new();
        for expr in exprs {
            let ExpressionData::BindingExpr(binding) = expr.data else {
                let span = expr.span();
                return Err(SerializationError::on_span(ExpectedBinding(expr), span));
            };

            // Wrap the value in a list so the iterator returns it as a single item
            let wrapper = Expression::list(vec![*binding.value]);
            ret.insert(binding.name, T::deserialize(&mut wrapper.into_iter(), source_text)?);
        }

        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::parser::Parser;
    use crate::serialization::EasyConfig;
    use std::collections::HashMap;

    // Vec tests
    #[test]
    fn serialize_vec() {
        assert_eq!(vec![1, 2, 3].serialize(), Expression::list(vec![
            Expression::presence(1),
            Expression::presence(2),
            Expression::presence(3),
        ]))
    }

    #[test]
    fn serialize_vec_empty() {
        let empty: Vec<i32> = vec![];
        assert_eq!(empty.serialize(), Expression::list(vec![]))
    }

    #[test]
    fn serialize_vec_strings() {
        assert_eq!(
            vec!["hello".to_string(), "world".to_string()].serialize(),
            Expression::list(vec![
                Expression::presence("hello".to_string()),
                Expression::presence("world".to_string()),
            ])
        )
    }

    #[test]
    fn deserialize_vec() {
        let source = vec![1, 2, 3].serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        assert_eq!(Vec::<i32>::deserialize(&mut parsed.into_iter(), source).unwrap(), vec![1, 2, 3])
    }

    #[test]
    fn deserialize_vec_empty() {
        let empty: Vec<i32> = vec![];
        let source = empty.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        assert_eq!(Vec::<i32>::deserialize(&mut parsed.into_iter(), source).unwrap(), vec![])
    }

    #[test]
    fn deserialize_vec_strings() {
        let original = vec!["foo".to_string(), "bar".to_string()];
        let source = original.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        assert_eq!(Vec::<String>::deserialize(&mut parsed.into_iter(), source).unwrap(), original)
    }

    #[test]
    fn vec_roundtrip() {
        let original = vec![10, 20, 30, 40, 50];
        let source = original.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = Vec::<i32>::deserialize(&mut parsed.into_iter(), source).unwrap();
        assert_eq!(deserialized, original)
    }

    #[test]
    fn vec_nested() {
        let nested = vec![vec![1, 2], vec![3, 4, 5], vec![]];
        let source = nested.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = Vec::<Vec<i32>>::deserialize(&mut parsed.into_iter(), source).unwrap();
        assert_eq!(deserialized, nested)
    }

    // Array tests
    #[test]
    fn serialize_array() {
        assert_eq!([1, 2, 3].serialize(), Expression::list(vec![
            Expression::presence(1),
            Expression::presence(2),
            Expression::presence(3),
        ]))
    }

    #[test]
    fn serialize_array_empty() {
        let empty: [i32; 0] = [];
        assert_eq!(empty.serialize(), Expression::list(vec![]))
    }

    #[test]
    fn deserialize_array() {
        let source = [1, 2, 3].serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        assert_eq!(<[i32; 3]>::deserialize(&mut parsed.into_iter(), source).unwrap(), [1, 2, 3])
    }

    #[test]
    fn deserialize_array_empty() {
        let empty: [i32; 0] = [];
        let source = empty.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        assert_eq!(<[i32; 0]>::deserialize(&mut parsed.into_iter(), source).unwrap(), [])
    }

    #[test]
    fn array_roundtrip() {
        let original = [5, 10, 15];
        let source = original.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = <[i32; 3]>::deserialize(&mut parsed.into_iter(), source).unwrap();
        assert_eq!(deserialized, original)
    }

    #[test]
    fn deserialize_array_wrong_cardinality_too_few() {
        let source = vec![1, 2].serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let result = <[i32; 3]>::deserialize(&mut parsed.into_iter(), source);
        assert!(result.is_err())
    }

    #[test]
    fn deserialize_array_wrong_cardinality_too_many() {
        let source = vec![1, 2, 3, 4].serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let result = <[i32; 3]>::deserialize(&mut parsed.into_iter(), source);
        assert!(result.is_err())
    }

    #[test]
    fn array_strings() {
        let original = ["a".to_string(), "b".to_string()];
        let source = original.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = <[String; 2]>::deserialize(&mut parsed.into_iter(), source).unwrap();
        assert_eq!(deserialized, original)
    }

    // HashMap tests
    #[test]
    fn serialize_hashmap() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), 42);
        map.insert("key2".to_string(), 100);

        let serialized = map.serialize();
        let source = serialized.dump();

        // Parse back to verify structure
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = HashMap::<String, i32>::deserialize(&mut parsed.into_iter(), source).unwrap();

        assert_eq!(deserialized.get("key1"), Some(&42));
        assert_eq!(deserialized.get("key2"), Some(&100));
    }

    #[test]
    fn serialize_hashmap_empty() {
        let map: HashMap<String, i32> = HashMap::new();
        assert_eq!(map.serialize(), Expression::list(vec![]))
    }

    #[test]
    fn deserialize_hashmap() {
        let mut original = HashMap::new();
        original.insert("foo".to_string(), 10);
        original.insert("bar".to_string(), 20);

        let source = original.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = HashMap::<String, i32>::deserialize(&mut parsed.into_iter(), source).unwrap();

        assert_eq!(deserialized.len(), 2);
        assert_eq!(deserialized.get("foo"), Some(&10));
        assert_eq!(deserialized.get("bar"), Some(&20));
    }

    #[test]
    fn deserialize_hashmap_empty() {
        let empty: HashMap<String, i32> = HashMap::new();
        let source = empty.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = HashMap::<String, i32>::deserialize(&mut parsed.into_iter(), source).unwrap();
        assert!(deserialized.is_empty())
    }

    #[test]
    fn hashmap_roundtrip() {
        let mut original = HashMap::new();
        original.insert("alpha".to_string(), "first".to_string());
        original.insert("beta".to_string(), "second".to_string());
        original.insert("gamma".to_string(), "third".to_string());

        let source = original.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = HashMap::<String, String>::deserialize(&mut parsed.into_iter(), source).unwrap();

        assert_eq!(deserialized.len(), original.len());
        for (key, value) in &original {
            assert_eq!(deserialized.get(key), Some(value));
        }
    }

    #[test]
    fn hashmap_with_vec_values() {
        let mut original = HashMap::new();
        original.insert("numbers".to_string(), vec![1, 2, 3]);
        original.insert("empty".to_string(), vec![]);
        original.insert("more".to_string(), vec![10, 20]);

        let source = original.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = HashMap::<String, Vec<i32>>::deserialize(&mut parsed.into_iter(), source).unwrap();

        assert_eq!(deserialized.get("numbers"), Some(&vec![1, 2, 3]));
        assert_eq!(deserialized.get("empty"), Some(&vec![]));
        assert_eq!(deserialized.get("more"), Some(&vec![10, 20]));
    }

    #[test]
    fn hashmap_nested_maps() {
        let mut inner1 = HashMap::new();
        inner1.insert("a".to_string(), 1);
        inner1.insert("b".to_string(), 2);

        let mut inner2 = HashMap::new();
        inner2.insert("c".to_string(), 3);

        let mut outer = HashMap::new();
        outer.insert("map1".to_string(), inner1.clone());
        outer.insert("map2".to_string(), inner2.clone());

        let source = outer.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = HashMap::<String, HashMap<String, i32>>::deserialize(&mut parsed.into_iter(), source).unwrap();

        assert_eq!(deserialized.get("map1"), Some(&inner1));
        assert_eq!(deserialized.get("map2"), Some(&inner2));
    }

    // Complex nested structures
    #[test]
    fn complex_nested_structure() {
        let mut map = HashMap::new();
        map.insert("arrays".to_string(), vec![
            vec![1, 2],
            vec![3, 4, 5],
        ]);

        let source = map.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = HashMap::<String, Vec<Vec<i32>>>::deserialize(&mut parsed.into_iter(), source).unwrap();

        assert_eq!(deserialized.get("arrays"), Some(&vec![vec![1, 2], vec![3, 4, 5]]));
    }

    #[test]
    fn vec_of_arrays() {
        let original = vec![[1, 2], [3, 4], [5, 6]];
        let source = original.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = Vec::<[i32; 2]>::deserialize(&mut parsed.into_iter(), source).unwrap();
        assert_eq!(deserialized, original)
    }

    #[test]
    fn array_of_vecs() {
        let original = [vec![1, 2, 3], vec![4, 5], vec![]];
        let source = original.serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap();
        let deserialized = <[Vec<i32>; 3]>::deserialize(&mut parsed.into_iter(), source).unwrap();
        assert_eq!(deserialized, original)
    }
}