pub trait StrExtensions<'a> {
    fn first(&self) -> Option<char>;
    fn take(&mut self, n: usize) -> &'a str;
    fn take_while(&mut self, condition: impl FnMut(char) -> bool) -> &'a str;
    fn skip(&mut self, n: usize) {
        self.take(n);
    }
}

impl<'a> StrExtensions<'a> for &'a str {
    fn first(&self) -> Option<char> {
        self.chars().next()
    }

    /// Takes up to `n` (ascii) characters, if there are enough elements in `self` then `n` will be
    /// fetched. If there are fewer than `n` all of `self` will be taken.
    fn take(&mut self, n: usize) -> &'a str {
        if let Some((left, right)) = self.split_at_checked(n) {
            *self = right;
            left
        } else {
            std::mem::replace(self, "")
        }
    }

    fn take_while(&mut self, mut condition: impl FnMut(char) -> bool) -> &'a str {
        let index = self.find(|c| !condition(c)).unwrap_or(self.len());
        let (before, after) = self.split_at(index);
        *self = after;
        before
    }
}