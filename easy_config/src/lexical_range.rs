#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LexicalSpan {
    start: usize,
    end: usize,
}

impl LexicalSpan {
    pub fn zeros() -> LexicalSpan {
        LexicalSpan { start: 0, end: 0 }
    }
    pub fn new(start: usize, end: usize) -> LexicalSpan {
        LexicalSpan { start, end }
    }
    
    pub fn start(&self) -> usize {
        self.start
    }
    
    pub fn end(&self) -> usize {
        self.end
    }
    pub fn combine(self, other: LexicalSpan) -> LexicalSpan {
        LexicalSpan::new(self.start().min(other.start()), self.end().max(other.end()))
    }
    
    pub fn slice<'a>(&'a self, item: &'a str) -> &'a str {
        if self.end <= self.start {
            ""
        } else {
            &item[self.start..self.end]
        }
    }
}

impl IntoIterator for LexicalSpan {
    type Item = usize;
    type IntoIter = core::ops::Range<usize>;
    fn into_iter(self) -> Self::IntoIter {
        (self.start..self.end).into_iter()
    }
}