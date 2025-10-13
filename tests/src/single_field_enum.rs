use easy_config_derive::EasyConfig;

#[derive(Debug, EasyConfig, PartialEq)]
enum TestEnum {
    First(String),
    Second(String),
}

#[cfg(test)]
mod tests {
    use easy_config::serialization::EasyConfig;
    use easy_config::expression::Expression;
    use easy_config::parser::Parser;
    use super::*;
    use pretty_assertions::assert_eq;
    use crate::single_field_enum::TestEnum::First;

    #[test]
    fn serialize() {
        assert_eq!(TestEnum::First("hello".to_string()).serialize().pretty(), Expression::list(vec![
            Expression::presence("First".to_string()),
            Expression::list(vec![Expression::presence("hello".to_string())])
        ]).pretty());
    }

    #[test]
    fn deserialize() {
        let text = First("hello".to_string()).serialize().dump();
        dbg!(&text);
        let x = TestEnum::deserialize(&mut Parser::new(&text).parse().unwrap().into_iter(), &text).unwrap();
        assert_eq!(x, TestEnum::First("hello".to_string()));
    }
}