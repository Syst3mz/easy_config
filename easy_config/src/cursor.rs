use std::iter::Peekable;
use crate::error::Error;
use crate::token::{Lexer, Token, TokenKind};

pub struct Cursor<'source> {
    inner: Peekable<Lexer<'source>>,
}

impl<'source> Cursor<'source> {
    pub fn new(source: &'source str) -> Cursor<'source> {
        Self {
            inner: Lexer::new(source).peekable(),
        }
    }
    pub fn peek(&mut self) -> Option<Result<Token, Error>> {
        self.inner.peek().map(|x| x.as_ref().map(|x| x.clone()).map_err(|x| x.clone()))
    }

    pub fn check(&mut self, kinds: &'static [TokenKind]) -> Result<Token, Error> {
        let peeked = self.peek().ok_or(Error::ReachedEoi)??;

        for kind in kinds {
            if kind == &peeked.kind {
                return Ok(peeked);
            }
        }

        Err(Error::UnexpectedToken(peeked.kind, kinds, peeked.span))
    }

    pub fn expect(&mut self, kinds: &'static [TokenKind]) -> Result<Token, Error> {
        let checked = self.check(kinds)?;
        self.inner.next();
        Ok(checked)
    }
}

impl<'a> Iterator for Cursor<'a> {
    type Item = Result<Token, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}