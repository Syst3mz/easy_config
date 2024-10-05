use easy_config_derive::Config;

#[derive(Config, PartialEq, Debug)]
enum Mode {
    Unit,
    TupleLike(f32, i32),
    StructLike {a: f32, b: i32},
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
    use super::*;
    use easy_config::parser::Parser;
    use easy_config::serialization::Config;

    #[test]
    fn serialize_unit() {
        let x = Mode::Unit;
        assert_eq!(x.serialize().dump(), "Unit")
    }

    #[test]
    fn serialize_tuple_like() {
        let x = Mode::TupleLike(2.0, 4);
        assert_eq!(x.serialize().dump(), "(TupleLike 2 4)")
    }

    #[test]
    fn serialize_struct_like() {
        let x = Mode::StructLike { a: 2.0, b: 4 };
        assert_eq!(x.serialize().dump(), "(StructLike a = 2 b = 4)")
    }

    #[test]
    fn deserialize_unit() {
        let text = Mode::Unit.serialize().dump();
        let parsed = Parser::new(text).parse().unwrap();
        assert_eq!(Mode::deserialize(parsed).unwrap(), Mode::Unit)
    }

    #[test]
    fn deserialize_tuple_like() {
        let text = Mode::TupleLike(2.0, 4).serialize().dump();
        let parsed = Parser::new(text).parse().unwrap();
        assert_eq!(Mode::deserialize(parsed).unwrap(), Mode::TupleLike(2.0, 4))
    }

    #[test]
    fn deserialize_struct_like() {
        let text = Mode::StructLike { a: 2.0, b: 4 }.serialize().dump();
        let parsed = Parser::new(text).parse().unwrap();
        assert_eq!(Mode::deserialize(parsed).unwrap(), Mode::StructLike { a: 2.0, b: 4 })
    }
}