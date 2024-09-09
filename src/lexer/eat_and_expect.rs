use std::iter::Peekable;

pub trait EatAndExpect<T> {
    fn eat(&mut self, comparer:  fn(&T) -> bool) -> Option<T>;
    fn expect(&mut self, comparer:  fn(&T) -> bool) -> Option<&T>;
}

impl<T: PartialEq, I: Iterator<Item=T>> EatAndExpect<T> for Peekable<I> {
    fn eat(&mut self, comparer: fn(&T) -> bool) -> Option<T> {
        if self.expect(comparer).is_some() {
            self.next()
        } else {
            None
        }
    }

    fn expect(&mut self, comparer:  fn(&T) -> bool) -> Option<&T> {
        let peeked = self.peek()?;
        if comparer(peeked) {
            return Some(peeked);
        }

        return None;
    }
}