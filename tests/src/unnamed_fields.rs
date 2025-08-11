use easy_config_derive::EasyConfig;

#[derive(EasyConfig)]
struct UnnamedFields(
    String,
    #[comment("My favorite numbers in order.")]
    Vec<u32>
);

#[allow(dead_code)]
fn testing() -> UnnamedFields {
    UnnamedFields("hello world".to_string(), vec![1, 2, 3])
}

#[cfg(test)]
mod tests {
    use crate::unnamed_fields::testing;
    use easy_config::serialization::EasyConfig;
    use easy_config::expression::Expression;

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
        todo!()
    }
}