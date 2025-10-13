#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::parser::Parser;
    use crate::serialization::EasyConfig;
    use crate::tests::EzStruct;

    fn demo() -> Vec<EzStruct> {
        vec![
            EzStruct {
                name: "James".to_string(),
                count: 4,
            },

            EzStruct {
                name: "Sammy".to_string(),
                count: 8,
            },
        ]
    }

    #[test]
    fn serialize_structs() {
        assert_eq!(demo().serialize(), Expression::list(vec![
            Expression::list(vec![
                Expression::presence("EzStruct"),
                Expression::list(vec![
                    Expression::binding("name", Expression::presence("James")),
                    Expression::binding("count", Expression::presence(4)),
                ])
            ]),
            Expression::list(vec![
                Expression::presence("EzStruct"),
                Expression::list(vec![
                    Expression::binding("name", Expression::presence("Sammy")),
                    Expression::binding("count", Expression::presence(8)),
                ])
            ]),
        ]))
    }

    #[test]
    fn deserialize() {
        let source = demo().serialize().dump();
        let expr = Parser::new(&source).parse().unwrap();
        assert_eq!(Vec::<EzStruct>::deserialize(&mut expr.into_iter(), source).unwrap(), demo())
    }

    #[test]
    fn deserialize_name() {
        let source = "(
            (name = James count = 4)
            (name = Sammy count = 8)
        )";
        let expr = Parser::new(&source).parse().unwrap();
        assert_eq!(Vec::<EzStruct>::deserialize(&mut expr.into_iter(), source).unwrap(), demo())
    }

    #[test]
    fn deserialize_partial_name() {
        let source = "(
            EzStruct (name = James count = 4)
            (name = Sammy count = 8)
        )";
        let expr = Parser::new(&source).parse().unwrap();
        assert_eq!(Vec::<EzStruct>::deserialize(&mut expr.into_iter(), source).unwrap(), demo())
    }

    #[test]
    fn deserialize_other_partial_name() {
        let source = "(
            (name = James count = 4)
            EzStruct (name = Sammy count = 8)
        )";
        let expr = Parser::new(&source).parse().unwrap();
        assert_eq!(Vec::<EzStruct>::deserialize(&mut expr.into_iter(), source).unwrap(), demo())
    }
}