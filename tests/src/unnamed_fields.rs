use easy_config_derive::EasyConfig;

#[derive(EasyConfig, Debug, PartialEq)]
struct UnnamedFields(
    String,
    #[comment("My favorite numbers in order.")]
    Vec<u32>,
);

#[allow(dead_code)]
fn testing() -> UnnamedFields {
    UnnamedFields("hello world".to_string(), vec![1, 2, 3])
}

#[cfg(test)]
mod tests {
    use crate::unnamed_fields::{testing, UnnamedFields};
    use easy_config::serialization::EasyConfig;
    use easy_config::expression::Expression;
    use easy_config::parser::Parser;

    #[test]
    fn serialize() {
        assert_eq!(testing().serialize(), Expression::list(vec![
            Expression::presence("UnnamedFields"),
            Expression::list(vec![
                Expression::presence("hello"),
                Expression::presence("world"),
            ]),
            Expression::list(vec![
                Expression::presence(1),
                Expression::presence(2),
                Expression::presence(3)
            ]).with_comment("My favorite numbers in order."),
        ]))
    }

    #[test]
    fn deserialize() {
        let exprs = testing().serialize();
        let text = exprs.uncomented_dump();
        let parsed = Parser::new(&text).parse().unwrap().into_iter().next().unwrap();
        println!("{}", text);
        let deserialized = UnnamedFields::deserialize(&mut parsed.into_iter(), text).expect("should deserialize");
        assert_eq!(deserialized, testing());
    }
}