use easy_config_derive::EasyConfig;

#[derive(Debug, EasyConfig, PartialEq)]
enum TestEnum {
    First(String),
    Second(String),
}

#[derive(Debug, EasyConfig, PartialEq)]
struct TestStruct {
    contents: Vec<TestEnum>
}

#[cfg(test)]
mod tests {
    use easy_config::expression::Expression;
    use easy_config::parser::Parser;
    use easy_config::serialization::EasyConfig;
    use super::*;
    use pretty_assertions::assert_eq;

    fn test_struct() -> TestStruct {
        TestStruct {
            contents: vec![
                TestEnum::First("hello".to_string()),
                TestEnum::Second("world".to_string())
            ],
        }
    }

    #[test]
    fn serialize() {
        assert_eq!(test_struct().serialize(), Expression::list(vec![
            Expression::presence("TestStruct"),
            Expression::list(vec![
                Expression::binding("contents", Expression::list(vec![
                    Expression::list(vec![Expression::presence("First"), Expression::list(vec![Expression::presence("hello")])]),
                    Expression::list(vec![Expression::presence("Second"), Expression::list(vec![Expression::presence("world")])])
                ]))
            ])
        ]))
    }

    #[test]
    fn deserialize() {
        let serialized_text = test_struct().serialize().uncomented_dump();
        let parsed = Parser::new(&serialized_text).parse().unwrap();
        let deserialized = TestStruct::deserialize(
            &mut parsed.into_iter().next_or_err().unwrap().into_iter(),
            serialized_text
        ).unwrap();
        assert_eq!(deserialized, test_struct())
    }
}