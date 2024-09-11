pub trait StrExtensions<'a> {
    fn first(&self) -> Option<char>;
    fn take(&mut self, n: usize) -> &'a str;
    fn take_while(&mut self, accept: impl FnMut(char) -> bool) -> &'a str;
    fn skip(&mut self, n: usize) {
        self.take(n);
    }
}

impl<'a> StrExtensions<'a> for &'a str{
    fn first(&self) -> Option<char> {
        self.chars().next()
    }

    fn take(&mut self, n: usize) -> &'a str {
        let (left, right) = self.split_at_checked(n)
            .expect(&format!("Unable to split string at {}", n));
        *self = right;
        left
    }

    fn take_while(&mut self, mut accept: impl FnMut(char) -> bool) -> &'a str {
        let index = self.find(|x| !accept(x)).unwrap_or(self.len());
        let (left, right) = self.split_at(index);
        *self = right;
        left
    }
}