use easy_config_derive::EasyConfig;

#[derive(Debug, EasyConfig, PartialEq)]
enum TestEnum {
    First(String),
    Second(String),
}

#[cfg(test)]
mod tests {
    use easy_config::expression::Expression;
    use easy_config::parser::Parser;
    use super::*;

    #[test]
    fn serialize() {
        assert_eq!(TestEnum::First("hello".to_string()).serialize(), Expression::list(vec![
            Expression::presence("First".to_string()),
            Expression::list(vec![Expression::presence("hello".to_string()),])
        ]));
    }

    #[test]
    fn deserialize() {
        let text = "(First (hello))";
        let x = TestEnum::deserialize(&mut Parser::new(text).parse().unwrap().into_iter(), text).unwrap();
        assert_eq!(x, TestEnum::First("hello".to_string()));
    }
}