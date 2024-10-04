use derive::Config;

#[derive(Config, Debug, PartialEq)]
struct Vec2 {
    x: f32,
    y: f32
}

#[cfg(test)]
mod tests {
    use easy_config::parser::Parser;
    use easy_config::serialization::Config;
    use super::*;

    #[test]
    fn serialize() {
        assert_eq!(Vec2 {x: 1.0, y: 2.0}.serialize().dump(), "(x = 1 y = 2)".to_string())
    }

    #[test]
    fn deserialize() {
        let text = Vec2 {x: 1.0, y: 2.0}.serialize().dump();
        let parsed = Parser::new(text).parse().unwrap();
        assert_eq!(Vec2::deserialize(parsed).unwrap(), Vec2 {x: 1.0, y: 2.0})
    }
}