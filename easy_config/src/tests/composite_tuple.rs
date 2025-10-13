#[cfg(test)]
mod tests {
    use crate::expression::Expression;
    use crate::parser::Parser;
    use crate::serialization::EasyConfig;
    use crate::tests::{ez_struct, EzStruct};

    #[test]
    fn serialization() {
        let x = (2, ez_struct()).serialize();
        assert_eq!(x, Expression::list(vec![
            Expression::presence(2),
            ez_struct().serialize()
        ]))
    }

    #[test]
    fn deserialization() {
        let source = (2, ez_struct()).serialize().dump();
        let parsed = Parser::new(&source).parse().unwrap().minimized();
        let got = <(i32, EzStruct)>::deserialize(&mut parsed.into_iter(), &source).unwrap();
        assert_eq!(got, (2, ez_struct()));
    }
}