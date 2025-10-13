#[cfg(test)]
mod simple_tests {
    use crate::expression::Expression;
    use crate::parser::Parser;
    use crate::serialization::EasyConfig;
    use crate::tests::{ez_struct, EzStruct};

    #[test]
    fn serialize() {
        let expected = Expression::list(vec![
            Expression::presence("EzStruct"),
            Expression::list(vec![
                Expression::binding("name", Expression::presence("Momo")),
                Expression::binding("count", Expression::presence(3))
            ])
        ]);

        assert_eq!(ez_struct().serialize(), expected);
    }

    #[test]
    fn deserialize() {
        let source = ez_struct().serialize().dump();
        let parsed = Parser::new(source.as_str()).parse().unwrap();
        assert_eq!(EzStruct::deserialize(&mut parsed.into_iter(), &source).unwrap(), ez_struct())
    }
}