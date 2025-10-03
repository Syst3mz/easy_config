#[cfg(test)]
mod simple_tests {
    use crate::expression::Expression;
    use crate::expression_iterator::ExpressionIterator;
    use crate::parser::Parser;
    use crate::serialization::EasyConfig;
    use crate::serialization::serialization_error::SerializationError;

    #[derive(Debug, PartialEq, Eq)]
    struct Demo {
        name: String,
        count: i32,
    }

    impl EasyConfig for Demo {
        const IS_STRUCT: bool = true;
        fn serialize(&self) -> Expression {
            Expression::list(vec![
                Expression::presence("Demo"),
                Expression::list(vec![
                    Expression::binding("name", Expression::presence(self.name.clone())),
                    Expression::binding("count", Expression::presence(self.count))
                ])
            ])
        }

        fn deserialize(expression_iterator: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError> where Self: Sized {
            let source_text = source_text.as_ref();
            let mut normalized = expression_iterator
                .normalized_struct("Demo")?
                .into_iter();
            normalized.next();
            let mut fields= normalized.binding_map()?;

            Ok(Self {
                name: fields.get("name", source_text)?,
                count: fields.get("count", source_text)?,
            })
        }
    }

    fn demo() -> Demo {
        Demo {
            name: "Momo".to_string(),
            count: 3,
        }
    }

    #[test]
    fn serialize() {
        let expected = Expression::list(vec![
            Expression::presence("Demo"),
            Expression::list(vec![
                Expression::binding("name", Expression::presence("Momo")),
                Expression::binding("count", Expression::presence(3))
            ])
        ]);

        assert_eq!(demo().serialize(), expected);
    }

    #[test]
    fn deserialize() {
        let source = demo().serialize().dump();
        let parsed = Parser::new(source.as_str()).parse().unwrap();
        assert_eq!(Demo::deserialize(&mut parsed.into_iter(), &source).unwrap(), demo())
    }
}