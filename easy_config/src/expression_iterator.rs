use std::collections::HashMap;
use std::iter::Peekable;
use crate::serialization::option_span_combine::OptionSpanCombine;
use crate::expression::{Atom, Binding, Expression};
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
    
    pub fn finished(&self) -> bool {
        self.len() == 0
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
    
    pub fn minimized_next_or_err(&mut self, source_text: impl AsRef<str>) -> Result<Expression, SerializationError> {
        self.next_or_err(source_text).map(|x| x.minimized())
    }
    pub fn next_list_or_err(&mut self, source_text: impl AsRef<str>) -> Result<Expression, SerializationError> {
        let source_text = source_text.as_ref();
        let expr = self.next_or_err(source_text)?;
        let span = expr.span();
        if !expr.is_list() {
            return Err(SerializationError::on_span(Kind::ExpectedList(expr), span, source_text))
        }
        
        Ok(expr)
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
    pub fn eat_presence_if_present(&mut self, value: impl Into<Atom>) -> bool {
        let Some(peek) = self.peek() else { return false };
        let Presence(peek, _) = &peek.data else { return false };
        let res = peek == &value.into();
        if res {
            self.next();
        }

        res
    }

    pub fn convert_binding_list_to_hashmap_of_values(&mut self, source_text: impl AsRef<str>) -> Result<(HashMap<String, Expression>, LexicalSpan), SerializationError> {
        let source_text = source_text.as_ref();
        let mut acc = HashMap::new();
        let mut outer_span = None;

        for item in self {
            let span = item.span();
            let BindingExpr(binding) = item.data else {
                return Err(SerializationError::on_span(
                    Kind::ExpectedBinding(item), span, source_text)
                    .contextualize("Expected a binding list to be comprised of exclusively bindings.")
                );
            };
            outer_span.combine(span);

            acc.insert(binding.name, *binding.value);
        }

        dbg!(&acc);

        Ok((acc, outer_span.ok_or(SerializationError::end_of_input(source_text))?))
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