use easy_config_derive::Config;

#[derive(Config, PartialEq, Debug)]
struct NeedsExplaining {
    #[EasyConfig(Comment = "A")]
    hard_to_understand_field: u32
}

#[cfg(test)]
mod tests {
    use easy_config::parser::Parser;
    use easy_config::serialization::Config;
    use super::*;

    #[test]
    fn serialize_with_comments() {
        assert_eq!(NeedsExplaining {hard_to_understand_field: 1}.serialize().pretty(), "# A\nhard_to_understand_field = 1")
    }

    #[test]
    fn deserialize_with_comments() {
        let serialized = NeedsExplaining {hard_to_understand_field: 1}.serialize().pretty();
        let deserialized = NeedsExplaining::deserialize(Parser::parse(serialized).unwrap()).unwrap();

        assert_eq!(deserialized, NeedsExplaining {hard_to_understand_field: 1})
    }
}