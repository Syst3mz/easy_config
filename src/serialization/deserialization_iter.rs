use std::collections::VecDeque;
use crate::parser::expression::Expression;

pub struct DeserializationIterator {
    internal: VecDeque<Expression>,
}

impl DeserializationIterator {
    pub fn new(internal: Vec<Expression>) -> DeserializationIterator {
        Self {
            internal: VecDeque::from(internal),
        }
    }
}

impl Iterator for DeserializationIterator {
    type Item = Expression;

    fn next(&mut self) -> Option<Self::Item> {
        self.internal.pop_front()
    }
}