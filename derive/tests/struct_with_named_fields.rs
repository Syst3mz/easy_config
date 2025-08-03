use easy_config::serialization::Config;
use easy_config_derive::Config;

#[derive(Config)]
struct NamedFields {
    x: String,
    #[Comment("My favorite numbers in order.")]
    z: Vec<u32>
}

fn testing() -> NamedFields {
    NamedFields {
        x: "hello world".to_string(),
        z: vec![1, 2, 3],
    }
}

#[cfg(test)]
mod tests {
    use easy_config::expression::Expression;
    use super::*;

    #[test]
    fn serialize() {
        assert_eq!(testing().serialize(), Expression::list(vec![
            Expression::binding("x", Expression::presence("hello world")),
            Expression::binding("z", Expression::list(vec![1, 2, 3]).with_comment("My favorite numbers in order.")),
        ]))
    }

    #[test]
    fn deserialize() {
        todo!()
    }
}