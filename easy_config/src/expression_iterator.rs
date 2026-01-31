use std::collections::HashMap;
use std::iter::Peekable;
use crate::binding_map::BindingMap;
use crate::config_error::Contextualize;
use crate::expression::{Atom, Expression};
use crate::expression::ExpressionData::{BindingExpr, List, Presence};
use crate::lexical_span::LexicalSpan;
use crate::serialization::EasyConfig;
use crate::serialization::serialization_error::{Kind, SerializationError};
use crate::serialization::serialization_error::Kind::{ExpectedBinding, ExpectedList, ExpectedPresence, ExpectedText};

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
    pub fn next_or_err(&mut self) -> Result<Expression, SerializationError> {
        self.next().ok_or(SerializationError::on_span(
            Kind::ReachedEoi,
            self.span().unwrap_or(LexicalSpan::new(0, 1)),
        ))
    }
    
    pub fn minimized_next_or_err(&mut self) -> Result<Expression, SerializationError> {
        self.next_or_err().map(|x| x.minimized())
    }
    pub fn next_list_or_err(&mut self) -> Result<Expression, SerializationError> {
        let expr = self.next_or_err()?;
        let span = expr.span();
        if !expr.is_list() {
            return Err(SerializationError::on_span(Kind::ExpectedList(expr), span))
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

    pub fn span(&self) -> Option<LexicalSpan> {
        self.spans.last().cloned()
    }
    pub fn peek(&mut self) -> Option<&Expression> {
        self.inner.peek()
    }
    pub fn eat_presence_if_present_and_matching(&mut self, value: impl Into<Atom>) -> bool {
        let Some(peek) = self.peek() else { return false };
        let Presence(peek, _) = &peek.data else { return false };
        let res = peek == &value.into();
        if res {
            self.next();
        }

        res
    }

    pub fn deserialize_next<T: EasyConfig>(&mut self, source_text: impl AsRef<str>) -> Result<T, SerializationError> {
        let next = self.next_or_err()?;
        let next_span = next.span();
        let next = Expression::list(vec![next]).with_span(next_span);
        T::deserialize(&mut next.into_iter(), source_text)
    }

    fn normalize_composite(name: String, exprs: Vec<Expression>, lexical_span: LexicalSpan, comment: Option<String>) -> Expression {
        Expression::list(vec![
            Expression::presence(Atom::Text(name)),
            Expression::new(List(exprs, lexical_span), comment),
        ])
    }

    pub fn normalized_struct(&mut self, name_of_struct: impl AsRef<str>) -> Result<Expression, SerializationError> {
        let next = self.next_or_err()?;
        let comment = next.comment;
        let struct_name = name_of_struct.as_ref();

        match next.data {
            BindingExpr(b) => {
                let span = b.span;
                Err(SerializationError::on_span(ExpectedList(Expression::new(BindingExpr(b), None)), span))
            }
            List(l, s) => {
                // Case: already looks like (Demo (...))
                if let Some(Expression { data: Presence(Atom::Text(name), ..), .. }) = l.first() {
                    if name == struct_name {
                        return Ok(Expression::new(List(l, s), comment));
                    }
                }
                // Otherwise: wrap into (Demo (...))
                Ok(Self::normalize_composite(struct_name.to_string(), l, s, comment))
            }
            Presence(p, s) => {
                let Atom::Text(name) = p else {
                    return Err(SerializationError::on_span(ExpectedText(p.to_string()), s));
                };

                let next = self.next_or_err()?;
                let List(list, list_span) = next.data else {
                    let span = next.span();
                    return Err(SerializationError::on_span(ExpectedList(next), span));
                };

                Ok(Self::normalize_composite(name, list, list_span, comment))
            }
        }
    }


    fn normalized_enum_helper(&mut self) -> Result<Expression, SerializationError> {
        let discriminant_expr = self.next_or_err()?;
        let discriminant_span = discriminant_expr.span();

        let Presence(discriminant, discriminant_span) = discriminant_expr.data else {
            return Err(SerializationError::on_span(
                ExpectedPresence(discriminant_expr),
                discriminant_span,
            ));
        };


        let discriminant_comment = discriminant_expr.comment;

        let Atom::Text(discriminant) = discriminant else {
            return Err(SerializationError::on_span(
                ExpectedText(discriminant.to_string()),
                discriminant_span,
            ));
        };

        // If there’s no payload, return empty composite
        let Some(peeked) = self.peek() else {
            return Ok(Self::normalize_composite(discriminant, vec![], discriminant_span, discriminant_comment));
        };

        // If the next expr isn’t a list, treat it as no payload
        if !peeked.is_list() {
            return Ok(Self::normalize_composite(discriminant, vec![], discriminant_span, discriminant_comment));
        }

        // Otherwise consume the list and treat its contents as the payload
        let next = self.next_or_err()?;
        let List(list, list_span) = next.data else {
            unreachable!()
        };

        Ok(Self::normalize_composite(discriminant, list, discriminant_span.combine(list_span), discriminant_comment))
    }

    pub fn normalized_enum(&mut self) -> Result<Expression, SerializationError> {
        let Some(expr) = self.peek() else {
            return Err(SerializationError::end_of_input())
                .contextualize("Failed to read enum.")
        };

        if expr.is_list() {
            let expr = self.next_or_err()?;
            let expr = &mut expr.into_iter();
            expr.normalized_enum_helper()
        } else {
            self.normalized_enum_helper()
        }
    }


    pub fn binding_map(&mut self) -> Result<BindingMap, SerializationError> {
        let next = self.next_or_err()?;
        let List(list, span) = next.data else {
            let span = next.span();
            return Err(SerializationError::on_span(ExpectedList(next), span))
        };

        let mut hashmap= HashMap::new();
        for expr in list {
            let BindingExpr(binding) = expr.data else {
                let span = expr.span();
                return Err(SerializationError::on_span(ExpectedBinding(expr), span))
            };
            hashmap.insert(binding.name.clone(), *binding.value);
        }

        Ok(BindingMap::new(hashmap, span))
    }

    pub fn next_text_or_err(&mut self) -> Result<(String, LexicalSpan), SerializationError> {
        let maybe_atom = self.next_or_err()?;
        let Presence(atom, atom_span) = maybe_atom.data else {
            let span = maybe_atom.span();
            return Err(SerializationError::on_span(ExpectedPresence(maybe_atom), span))
        };

        let Atom::Text(discriminant) = atom else {
            return Err(SerializationError::on_span(ExpectedText(atom.to_string()), atom_span))
        };

        Ok((discriminant, atom_span))
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_complete_struct() {
        let expected = Expression::list(vec![
            Expression::presence("name"),
            Expression::list(vec![Expression::presence("a"), Expression::presence("b")])
        ]);

        let mut expected_iter = expected.clone().into_iter();

        assert_eq!(expected_iter.normalized_struct("name").unwrap(), expected);
    }

    #[test]
    fn normalize_incomplete_struct() {
        let expected = Expression::list(vec![
            Expression::presence("name"),
            Expression::list(vec![Expression::presence("a"), Expression::presence("b")])
        ]);

        let mut incomplete_iter = Expression::list(vec![
            Expression::list(vec![Expression::presence("a"), Expression::presence("b")])
        ]).clone().into_iter();

        assert_eq!(incomplete_iter.normalized_struct("name").unwrap(), expected);
    }

    #[test]
    fn normalize_complete_enum() {
        // Input looks like: (name (a b))
        let input = Expression::list(vec![
            Expression::presence("name"),
            Expression::list(vec![Expression::presence("a"), Expression::presence("b")]),
        ]);

        let mut iter = input.into_iter();

        let mut expr = iter.normalized_enum().unwrap().into_iter();
        let name = expr.next_or_err().unwrap();
        assert_eq!(name.data, Expression::presence("name").data);
        assert_eq!(
            expr.next_or_err().unwrap().data,
            Expression::list(vec![
                Expression::presence("a"),
                Expression::presence("b"),
            ]).data
        );
    }

    #[test]
    fn normalize_incomplete_enum() {
        // Input looks like: (name)
        let input = Expression::list(vec![Expression::presence("name")]);

        let mut iter = input.into_iter();

        let mut expr = iter.normalized_enum().unwrap().into_iter();

        assert_eq!(expr.next_or_err().unwrap().data, Expression::presence("name").data);
        assert_eq!(expr.next_or_err().unwrap().data, Expression::list(vec![]).data);
    }


    #[test]
    fn normalize_struct_does_not_double_wrap() {
        // Already looks like (Demo (...))
        let already_normalized = Expression::list(vec![
            Expression::presence("Demo"),
            Expression::list(vec![
                Expression::binding("name", Expression::presence("Momo")),
                Expression::binding("count", Expression::presence(3)),
            ]),
        ]);

        // Running through normalized_struct again should be idempotent
        let mut iter = already_normalized.clone().into_iter();
        let normalized = iter.normalized_struct("Demo").unwrap();

        assert_eq!(normalized, already_normalized);
    }
}