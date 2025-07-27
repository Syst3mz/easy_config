use crate::config_error::Contextualize;
use crate::expression::{Atom, ExpressionData};
use crate::expression_iterator::ExpressionIterator;
use crate::serialization::serialization_error::{Kind, SerializationError};


pub fn get_discriminant_lowercased(expression_iterator: &mut ExpressionIterator, options: &'static [&'static str], source_text: impl AsRef<str>) -> Result<String, SerializationError> {
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

    let discriminant = discriminant.to_lowercase();
    if options.contains(&discriminant.as_str()) {
        Ok(discriminant)
    } else {
        Err(SerializationError::on_span(Kind::ExpectedDiscriminant(discriminant, options), discriminant_span, source_text))
    }
}