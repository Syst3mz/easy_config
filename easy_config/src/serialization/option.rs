use crate::config_error::Contextualize;
use crate::expression::Expression;
use crate::expression_iterator::ExpressionIterator;
use crate::serialization::EasyConfig;
use crate::serialization::serialization_error::{Kind, SerializationError};

impl<T: EasyConfig> EasyConfig for Option<T> {
    fn serialize(&self) -> Expression {
        match self {
            None => Expression::presence("None"),
            Some(s) => Expression::list(vec![Expression::presence("Some"), Expression::list(vec![s.serialize()])])
        }
    }

    fn deserialize(expression_iterator: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let source_text= source_text.as_ref();
        let (discriminant, discriminant_span, fields) = expression_iterator
            .normalized_enum()
            .contextualize("unable to deserialize enum")?;

        if discriminant == "None" {
            return Ok(None);
        }

        if discriminant == "Some" {
            // fields is Expression::list([ value ])
            let mut fields_iter = fields.into_iter();
            let inner = fields_iter.next_or_err()?.minimized();

            return Ok(Some(T::deserialize(&mut inner.into_iter(), source_text)?));
        }

        Err(SerializationError::on_span(Kind::ExpectedDiscriminant(discriminant, &["None", "Some"]), discriminant_span))
    }
}



#[cfg(test)]
mod tests {
    use crate::serialization::{EasyConfig, Expression};
    use crate::serialization::serialization_error::Kind;

    #[test]
    fn serialize_none() {
        let x: Option<i32> = None;
        let expr = x.serialize();
        assert_eq!(expr, Expression::presence("None"));

        // Round-trip through deserialize
        let got = <Option<i32>>::deserialize(&mut expr.clone().into_iter(), "").unwrap();
        assert_eq!(got, None);
    }

    #[test]
    fn serialize_some_primitive() {
        let x: Option<i32> = Some(42);
        let expr = x.serialize();
        let source = x.serialize().dump();
        assert_eq!(
            expr,
            Expression::list(vec![
                Expression::presence("Some"),
                Expression::list(vec![Expression::presence(42)])
            ])
        );

        let got = <Option<i32>>::deserialize(&mut expr.clone().into_iter(), source).unwrap();
        assert_eq!(got, Some(42));
    }

    #[test]
    fn invalid_discriminant() {
        // Something like (Maybe (123)) should fail
        let expr = Expression::list(vec![
            Expression::presence("Maybe"),
            Expression::list(vec![Expression::presence(123)])
        ]);

        let err = <Option<i32>>::deserialize(&mut expr.into_iter(), "").unwrap_err();
        match err.kind() {
            Kind::ExpectedDiscriminant(found, expected) => {
                assert_eq!(found, "Maybe");
                assert_eq!(expected, &["None", "Some"]);
            }
            _ => panic!("wrong error kind: {:?}", err),
        }
    }
}
