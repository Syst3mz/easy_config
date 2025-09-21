use easy_config_derive::EasyConfig;

#[derive(EasyConfig, PartialEq, Debug, Clone)]
struct NamedFields {
    x: String,
    #[comment("My favorite numbers in order.")]
    z: Vec<u32>
}

#[allow(dead_code)]
fn testing() -> NamedFields {
    NamedFields {
        x: "hello world".to_string(),
        z: vec![1, 2, 3],
    }
}

#[cfg(test)]
mod tests {
    use easy_config::expression::Expression;
    use easy_config::parser::Parser;
    use easy_config::serialization::EasyConfig;

    use super::*;

    #[test]
    fn serialize() {

        assert_eq!(testing().serialize(), Expression::list(vec![
            Expression::presence("NamedFields"),
            Expression::binding("x", Expression::list(vec![
                Expression::presence("hello"),
                Expression::presence("world"),
            ])),
            Expression::binding("z", Expression::list(vec![
                Expression::presence(1),
                Expression::presence(2),
                Expression::presence(3)
            ])).with_comment("My favorite numbers in order."),
        ]))
    }

    #[test]
    fn deserialize() {
        let exprs = testing().serialize();
        let text = exprs.uncomented_dump();
        let parsed = Parser::new(&text).parse().unwrap().into_iter().next().unwrap();
        println!("{}", text);
        let deserialized = NamedFields::deserialize(&mut parsed.into_iter(), text).expect("should deserialize");
        assert_eq!(deserialized, testing());
    }
}