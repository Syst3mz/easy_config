use easy_config_derive::EasyConfig;

#[derive(Debug, Clone, Copy, EasyConfig, PartialEq)]
enum Complex {
    Unit,
    Named {
        x: u32,
        y: i32,
    },
    Unnamed(u32, i32)
}


#[cfg(test)]
mod tests {
    use easy_config::serialization::EasyConfig;
    use easy_config::expression::Expression;
    use easy_config::parser::Parser;
    use super::*;

    #[test]
    fn unit_serialize() {
        assert_eq!(Complex::Unit.serialize(), Expression::list(vec![
            Expression::presence("Unit"),
            Expression::list(vec![])
        ]))
    }

    #[test]
    fn named_serialize() {
        assert_eq!(Complex::Named { x: 1, y: 2 }.serialize(), Expression::list(vec![
            Expression::presence("Named"),
            Expression::list(vec![
                Expression::binding("x", Expression::presence(1)),
                Expression::binding("y", Expression::presence(2))
            ])
        ]))
    }

    #[test]
    fn unnamed_serialize() {
        assert_eq!(Complex::Unnamed(3, 4).serialize(), Expression::list(vec![
            Expression::presence("Unnamed"),
            Expression::list(vec![
                Expression::presence(3),
                Expression::presence(4),
            ])
        ]))
    }

    #[test]
    fn unit_deserialize() {
        let text = "(Unit)";
        let result = Complex::deserialize(&mut Parser::new(text).parse().unwrap().into_iter(), text).unwrap();
        assert_eq!(result, Complex::Unit);
    }

    #[test]
    fn named_deserialize() {
        let text = "(Named (x=1 y=2))";
        let result = Complex::deserialize(&mut Parser::new(text).parse().unwrap().into_iter(), text).unwrap();
        assert_eq!(result, Complex::Named { x: 1, y: 2 });
    }

    #[test]
    fn unnamed_deserialize() {
        let text = "(Unnamed (3 4))";
        let result = Complex::deserialize(&mut Parser::new(text).parse().unwrap().into_iter(), text).unwrap();
        assert_eq!(result, Complex::Unnamed(3, 4));
    }
}