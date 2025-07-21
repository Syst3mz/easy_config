#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LexicalSpan {
    start: usize,
    end: usize,
}

impl LexicalSpan {
    pub fn new(start: usize, end: usize) -> LexicalSpan {
        LexicalSpan { start, end }
    }
    
    pub fn start(&self) -> usize {
        self.start
    }
    
    pub fn end(&self) -> usize {
        self.end
    }
}

impl IntoIterator for LexicalSpan {
    type Item = usize;
    type IntoIter = core::ops::Range<usize>;
    fn into_iter(self) -> Self::IntoIter {
        (self.start..self.end).into_iter()
    }
}