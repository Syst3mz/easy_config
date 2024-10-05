use easy_config_derive::Config;

#[derive(Config, Debug, PartialEq)]
struct Point(f32, f32);

#[cfg(test)]
mod tests {
    use easy_config_core::parser::Parser;
    use super::*;

    fn point() -> Point {
        Point(2.0, -4.0)
    }

    #[test]
    fn serialize() {
        assert_eq!(point().serialize().dump(), "(2 -4)".to_string())
    }

    #[test]
    fn deserialize() {
        let text = point().serialize().dump();
        let parsed = Parser::new(text).parse().unwrap();

        assert_eq!(Point::deserialize(parsed).unwrap(), point())
    }
}