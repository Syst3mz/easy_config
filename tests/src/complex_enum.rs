use easy_config_derive::EasyConfig;

#[derive(Debug, Clone, Copy, EasyConfig)]
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
    use easy_config::expression::Expression;
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
}