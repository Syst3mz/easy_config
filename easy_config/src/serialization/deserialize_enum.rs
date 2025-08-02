use crate::config_error::Contextualize;
use crate::expression::{Atom, ExpressionData};
use crate::expression_iterator::ExpressionIterator;
use crate::serialization::serialization_error::{Kind, SerializationError};

pub fn get_discriminant(expression_iterator: &mut ExpressionIterator, options: &'static [&'static str], source_text: impl AsRef<str>) -> Result<String, SerializationError> {
    let source_text = source_text.as_ref();
    let discriminant = expression_iterator
        .next_or_err(source_text)
        .contextualize("Expected an enum discriminant, but ran into the end of the input.")?;

    println!("'{}'", discriminant.dump());
    let discriminant_span = discriminant.span();
    let ExpressionData::Presence(discriminant, _) = discriminant.data else {
        return Err(SerializationError::on_span(Kind::ExpectedPresence(discriminant), discriminant_span, source_text))
            .contextualize("Error while deserializing discriminant.")
    };
    let Atom::Text(discriminant) = discriminant else {
        return Err(SerializationError::on_span(Kind::ExpectedText(discriminant.to_string()), discriminant_span, source_text))
            .contextualize("Error while deserializing discriminant.")?;
    };

    let discriminant = discriminant;
    if options.contains(&discriminant.as_str()) {
        Ok(discriminant)
    } else {
        Err(SerializationError::on_span(Kind::ExpectedDiscriminant(discriminant, options), discriminant_span, source_text))
    }
}
pub fn deserialize_enum<T,F>(expression_iterator: &mut ExpressionIterator, options: &'static[&'static str], source_text: impl AsRef<str>, mut deserializer: F) -> Result<T, SerializationError>
where F: FnMut(&str, &mut ExpressionIterator, &str) -> Result<T, SerializationError>
{
    let source_text = source_text.as_ref();

    if let Some(expr) = expression_iterator.peek() {
        if expr.is_list() {
            dbg!(expr.dump());
            let mut iter = expression_iterator.next().unwrap().into_iter();
            let discriminant = get_discriminant(&mut iter, options, source_text)?;
            return deserializer(discriminant.as_str(), &mut iter, source_text);
        }
    }

    let discriminant = get_discriminant(expression_iterator, options, source_text)?;
    deserializer(discriminant.as_str(), expression_iterator, source_text)
}