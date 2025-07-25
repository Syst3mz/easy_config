use crate::expression::Expression;
use crate::expression::ExpressionData::{Binding, List, Presence};
use crate::lexical_span::LexicalSpan;
use crate::serialization::serialization_error::{Kind, SerializationError};

#[derive(Debug, Clone)]
pub struct ExpressionIterator {
    inner: std::vec::IntoIter<Expression>
}
impl ExpressionIterator {
    pub fn new(expr: Expression) -> Self {
        Self {
            inner: match expr.data {
                List(p, _) => p.into_iter(),
                Presence(_, _) | Binding(_, _, _) => vec![expr].into_iter(),
            },
        }
    }
    pub fn next_field(&mut self, field: impl AsRef<str>, source_text: impl AsRef<str>) -> Result<Expression, SerializationError> {
        let source_text = source_text.as_ref();
        let len_of_text = source_text.len();
        self.next().ok_or(SerializationError::on_span(
            Kind::ExpectedFieldGotEoi(field.as_ref().to_string()),
            LexicalSpan::new(len_of_text - 1, len_of_text),
            source_text 
        ))
    }
}

impl Iterator for ExpressionIterator {
    type Item = Expression;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
impl ExactSizeIterator for ExpressionIterator {
    fn len(&self) -> usize {
        self.inner.len()
    }
}
impl DoubleEndedIterator for ExpressionIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}