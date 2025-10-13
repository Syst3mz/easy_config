use std::collections::HashMap;
use crate::expression::Expression;
use crate::lexical_span::LexicalSpan;
use crate::serialization::EasyConfig;
use crate::serialization::serialization_error::Kind::MissingField;
use crate::serialization::serialization_error::SerializationError;

#[derive(Debug, Clone)]
pub struct BindingMap {
    hashmap: HashMap<String, Expression>,
    span: LexicalSpan
}

impl BindingMap {
    pub fn new(hashmap: HashMap<String, Expression>, span: LexicalSpan) -> Self {
        Self { hashmap, span }
    }
    pub fn get<T: EasyConfig>(&mut self, field: impl AsRef<str>, source_text: impl AsRef<str>) -> Result<T, SerializationError> {
        let field = field.as_ref();

        let expr = self.hashmap
            .remove(field)
            .ok_or(SerializationError::on_span(MissingField("name".to_string()), self.span))?;
        let expr_span = expr.span();
        T::deserialize(&mut Expression::list(vec![expr]).with_span(expr_span).into_iter(), source_text)
    }
}