use derive::Config;
use easy_config::parser::expression::Expression;
use easy_config::serialization::Config;

#[derive(Config, PartialEq, Debug)]
enum Mode {
    First,
    /*Second(f32),
    Third(f32, i32),
    Fourth {a: f32, b: i32}*/
    Fifth
}

// impl Config for Mode {
//     fn serialize(&self) -> Expression {
//         match self {
//             Mode::First => Expression::Presence("First".to_string()),
//             Mode::Second(f) => Expression::Collection(vec![
//                 Expression::Presence("Second".to_string()),
//                 Expression::Presence(f.to_string()),
//             ]),
//             Mode::Third { a, b } => Expression::Collection(vec![
//                 Expression::Presence("Third".to_string()),
//                 Expression::Pair("a".to_string(), Box::new(Expression::Presence(a.to_string()))),
//                 Expression::Pair("b".to_string(), Box::new(Expression::Presence(a.to_string()))),
//             ])
//         }
//     }
//
//     fn deserialize(expr: Expression) -> Result<Self, Error>
//     where
//         Self: Sized
//     {
//         todo!()
//     }
// }

#[cfg(test)]
mod tests {
    use easy_config::parser::Parser;
    use super::*;

    #[test]
    fn serialize_mode_one() {
        let x = Mode::First;
        assert_eq!(x.serialize().dump(), "First")
    }

    #[test]
    fn serialize_mode_five() {
        let x = Mode::Fifth;
        assert_eq!(x.serialize().dump(), "Fifth")
    }

    #[test]
    fn deserialize_mode_one() {
        let text = Mode::First.serialize().dump();
        let parsed = Parser::new(text).parse().unwrap();
        assert_eq!(Mode::deserialize(parsed).unwrap(), Mode::First)
    }

    #[test]
    fn deserialize_mode_five() {
        let text = Mode::Fifth.serialize().dump();
        let parsed = Parser::new(text).parse().unwrap();
        assert_eq!(Mode::deserialize(parsed).unwrap(), Mode::Fifth)
    }

    /*#[test]
    fn serialize_mode_two() {
        let x = Mode::Second(3.0);
        assert_eq!(x.serialize().dump(), "(Second 3)")
    }

    #[test]
    fn serialize_mode_three() {
        let x = Mode::Third(1.0, 2);
        assert_eq!(x.serialize().dump(), "(Third 1 2)")
    }

    #[test]
    fn serialize_mode_fourth() {
        let x = Mode::Fourth {
            a: 1.0,
            b: 2,
        };
        assert_eq!(x.serialize().dump(), "(Fourth a = 1 b = 2)")
    }*/
}