use std::iter::Peekable;
use crate::serialization::option_span_combine::OptionSpanCombine;
use crate::expression::{Binding, Expression};
use crate::expression::ExpressionData::{BindingExpr, List, Presence};
use crate::lexical_span::LexicalSpan;
use crate::serialization::serialization_error::{Kind, SerializationError};

#[derive(Debug, Clone)]
pub struct ExpressionIterator {
    inner: Peekable<std::vec::IntoIter<Expression>>,
    spans: Vec<LexicalSpan>,
}
impl ExpressionIterator {
    pub fn new(expr: Expression) -> Self {
        Self {
            inner: match expr.data {
                List(p, _) => p.into_iter().peekable(),
                Presence(_, _) | BindingExpr(_) => vec![expr].into_iter().peekable(),
            },
            spans: vec![],
        }
    }
    pub fn rewind(&mut self, by: usize) {
        if by == 0 {
            return
        }

        for _ in 0..by {
            _ = self.next_back()
        }
    }
    pub fn next_or_err(&mut self, source_text: impl AsRef<str>) -> Result<Expression, SerializationError> {
        let source_text = source_text.as_ref();
        let len_of_text = source_text.len();
        self.next().ok_or(SerializationError::on_span(
            Kind::ReachedEoi,
            LexicalSpan::new(len_of_text - 1, len_of_text),
            source_text
        ))
    }

    pub fn update_span(&mut self, span: LexicalSpan) {
        if self.spans.is_empty() {
            self.spans.push(span);
            return
        }

        let Some(last_span) = self.spans.last() else { return };
        self.spans.push(last_span.combine(span));
    }

    pub fn find_binding(&mut self, name: impl AsRef<str>, source_text: impl AsRef<str>) -> Result<Binding, SerializationError> {
        let mut span = None;
        let name = name.as_ref();

        let self_clone = self.inner.clone();

        let ret = self
            .find_map(|x| {
                span.combine(x.span());
                let BindingExpr(binding) = x.data else { return None };
                if binding.name != name {
                    None
                } else {
                    Some(binding)
                }
            })

            .ok_or(SerializationError::on_span(
                Kind::MissingField(name.to_string()),
                span.unwrap(),
                source_text
            ));
        self.inner = self_clone;
        ret
    }

    pub fn span(&self) -> Option<LexicalSpan> {
        self.spans.last().cloned()
    }
    pub fn peek(&mut self) -> Option<&Expression> {
        self.inner.peek()
    }
}

impl Iterator for ExpressionIterator {
    type Item = Expression;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.inner.next();
        ret.map(|x|{
            self.update_span(x.span());
            x
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}
impl ExactSizeIterator for ExpressionIterator {
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl DoubleEndedIterator for ExpressionIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.spans.pop();
        self.inner.next_back()
    }
}