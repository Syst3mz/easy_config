#[cfg(test)]
mod tests {
    use easy_config::config::Config;
use easy_config::parser::Parser;
use easy_config::parser::expression::Expression;
use derive::Serialize;
    use super::*;

    #[derive(Serialize)]
    struct Demo {
        key: String,
        vec: Vec<String>,
    }

    fn demo() -> Demo {
        Demo {
            key: "cat".to_string(),
            vec: vec!["bird".to_string(), "dog".to_string()],
        }
    }

    #[test]
    fn serialize() {
        let d = demo();
        assert_eq!(d.serialize().dump(), "(key = cat vec = (bird dog))")
    }

    // #[test]
    // fn deserialize() {
    //     let parsed = Parser::new(demo().serialize().dump()).parse().unwrap();
    //     assert_eq!(
    //         Demo::deserialize(parsed).unwrap(), demo()
    //     )
    // }
}