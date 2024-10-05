use derive::Config;

#[derive(Config, Debug, PartialEq)]
struct Unit;

#[cfg(test)]
mod tests {
    use core::parser::Parser;
    use super::*;

    #[test]
    fn serialize() {
        let x = Unit;
        assert_eq!(x.serialize().dump(), "Unit")
    }

    #[test]
    fn deserialize() {
        let text = Unit.serialize().dump();
        let parsed = Parser::new(text).parse().unwrap();
        assert_eq!(Unit::deserialize(parsed).unwrap(), Unit)
    }
}