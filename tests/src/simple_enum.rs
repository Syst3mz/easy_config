use easy_config_derive::EasyConfig;

#[derive(Debug, Copy, Clone, EasyConfig)]
enum EnumNoArgs {
    One,
}

#[cfg(test)]
mod tests {
    use easy_config::serialization::EasyConfig;
    use easy_config::expression::Expression;
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
        todo!()
    }
}