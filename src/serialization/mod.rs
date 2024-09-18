mod primitives;
mod deserialize;
mod error;
mod deserialization_iter;

use crate::parser::expression::Expression;

trait Serialize: 'static {
    fn serialize(&self) -> Expression;
}



#[cfg(test)]
mod tests {
    use crate::parser::Parser;
    use crate::serialization::deserialize::{Deserialize, DeserializeExtension};
    use crate::serialization::error::Error;
    use super::*;

    #[derive(Debug, PartialEq)]
    struct Demo {
        key: String,
        vec: Vec<String>,
    }

    impl Serialize for Demo {
        fn serialize(&self) -> Expression {
            Expression::Collection(vec![
                Expression::Pair("key".to_string(), Box::new(self.key.serialize())),
                Expression::Pair("vec".to_string(), Box::new(self.vec.serialize()))
            ])
        }
    }

    impl Deserialize for Demo {
        fn deserialize(expr: Expression) -> Result<Self, Error> where Self: Sized
        {
            Ok(Self {
                key: String::deserialize(expr.deserialize_get("key")?)?,
                vec: Vec::<String>::deserialize(expr.deserialize_get("vec")?)?
            })
        }
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

    #[test]
    fn deserialize() {
        let parsed = Parser::new(demo().serialize().dump()).parse().unwrap();
        assert_eq!(
            Demo::deserialize(parsed).unwrap(), demo()
        )
    }
}