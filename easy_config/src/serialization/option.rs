use crate::config_error::Contextualize;
use crate::expression::{Atom, Expression, ExpressionData};
use crate::expression_iterator::ExpressionIterator;
use crate::serialization::Config;
use crate::serialization::serialization_error::{Kind, SerializationError};
use crate::serialization::serialization_error::Kind::ExpectedList;

impl<T: Config> Config for Option<T> {
    fn serialize(&self) -> Expression {
        match self {
            None => Expression::presence("None".to_string()),
            Some(t) => Expression::list(vec![
                Expression::presence("Some"),
                Expression::list(vec![t.serialize().minimized()]),
            ]),
        }
    }

    fn deserialize(expression_iterator: &mut ExpressionIterator, source_text: impl AsRef<str>) -> Result<Self, SerializationError>
    where
        Self: Sized
    {
        let source_text = source_text.as_ref();
        let name = expression_iterator.next_or_err(source_text).contextualize("Expected the start of an Option, but ran into the end of input instead.")?;
        let name_span = name.span();

        let ExpressionData::Presence(name, _) = name.data else {
            return Err(SerializationError::on_span(Kind::ExpectedPresence(name), name_span, source_text))
        };

        let Atom::Text(name) = name else {
            return Err(SerializationError::on_span(Kind::ExpectedText(name.to_string()), name_span, source_text))
        };

        if name.to_lowercase() == "none" {
            return Ok(None)
        }

        if name.to_lowercase() != "some" {
            return Err(SerializationError::on_span(
                Kind::ExpectedDiscriminant(name, &["Some", "None"]),
                name_span,
                source_text
            ));
        }

        let args = expression_iterator.next_or_err(source_text)
            .contextualize("Expected an enum value, but ran into the end of input instead.")?;
        let args_span = args.span();
        if !args.is_list() {
            return Err(SerializationError::on_span(ExpectedList(args), args_span, source_text));
        }

        Ok(Some(T::deserialize(&mut args.into_iter(), source_text)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_some() {
        let target = Some(1);
        assert_eq!(target.serialize(), Expression::list(
            vec![
                Expression::presence("Some"),
                Expression::list(vec![Expression::presence(1)]),
            ],
        ));
    }
    #[test]
    fn deserialize_some() {
        let expected = Some(1);
        let got = Option::deserialize(&mut expected.serialize().into_iter(), "Some (1)").unwrap();
        assert_eq!(got, expected);
    }
    #[test]
    fn serialize_none() {
        let target: Option<String> = None;
        assert_eq!(target.serialize(), Expression::presence("None"));
    }

    #[test]
    fn deserialize_none() {
        let expected: Option<i32> = None;
        let got = Option::deserialize(&mut expected.serialize().into_iter(), "None").unwrap();
        assert_eq!(got, expected);
    }
}