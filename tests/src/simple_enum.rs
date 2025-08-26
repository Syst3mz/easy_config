use easy_config_derive::EasyConfig;

#[derive(Debug, Copy, Clone, EasyConfig, PartialEq)]
enum EnumNoArgs {
    One,
    Two
}

#[cfg(test)]
mod tests {
    use easy_config::serialization::EasyConfig;
    use easy_config::expression::Expression;
    use easy_config::parser::Parser;
    use crate::simple_enum::EnumNoArgs;

    #[test]
    fn serialize() {
        assert_eq!(EnumNoArgs::One.serialize(), Expression::list(vec![
            Expression::presence("One"),
            Expression::list(vec![])
        ]))
    }

    #[test]
    fn deserialize() {
        let exprs = EnumNoArgs::One.serialize();
        let text = exprs.uncomented_dump();
        let parsed = Parser::new(&text).parse().unwrap().into_iter().next().unwrap();
        println!("{}", text);
        let deserialized = EnumNoArgs::deserialize(&mut parsed.into_iter(), text).expect("should deserialize");
        assert_eq!(deserialized, EnumNoArgs::One)
    }
}