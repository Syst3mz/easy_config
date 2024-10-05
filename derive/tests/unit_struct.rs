use easy_config_derive::Config;

#[derive(Config, Debug, PartialEq)]
struct Unit;

#[cfg(test)]
mod tests {
    use easy_config::parser::Parser;
    use easy_config::serialization::Config;
    use super::*;


    #[test]
    fn serialize() {
        let x = Unit;
        assert_eq!(x.serialize().dump(), "Unit")
    }

    #[test]
    fn deserialize() {
        let text = Unit.serialize().dump();
        let parsed = Parser::new(text).parse_tokens().unwrap();
        assert_eq!(Unit::deserialize(parsed).unwrap(), Unit)
    }
}