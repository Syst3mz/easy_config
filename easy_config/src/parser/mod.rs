use crate::expression::{CstData, CstExpression};
use crate::lexer::{Lexer, token};
use crate::lexer::token::{Kind, Token};
use crate::lexer::token::Kind::Text;
use crate::location::Location;
use crate::parser::parser_error::ParserError;

pub mod parser_error;

pub struct Parser {
    tokens: Vec<Token>,
    current_index: usize
}
impl Parser {
    pub fn new(text: impl AsRef<str>) -> Parser {
        Self {
            tokens: Lexer::new(text.as_ref()).collect(),
            current_index: 0,
        }
    }

    fn advance(&mut self) {
        self.current_index += 1;
    }

    fn finished(&self) -> bool {
        self.current_index == self.tokens.len()
    }

    fn current(&self) -> Option<Token> {
        self.tokens.get(self.current_index).map(|x| x.clone())
    }

    fn eat(&mut self, kind: token::Kind) -> Option<Token> {
        let t = self.expect(kind);
        if t.is_some() {
            self.advance();
        }
        t
    }

    fn expect(&self, kind: token::Kind) -> Option<Token> {
        let token = self.current()?;
        if token.kind() == kind {
            Some(token)
        } else {
            None
        }
    }

    fn parse_pair(&mut self) -> Result<CstExpression, ParserError> {
        let key_token = self.current().ok_or(ParserError::eoi(&self))?;
        if key_token.kind() != Text {
            return Err(ParserError::expected_text(&key_token))
        }
        self.advance();

        let key_text = key_token.lexeme().to_string();

        if self.eat(Kind::Equals).is_some() {
            Ok(key_token.to_cst_expr(CstData::Pair(key_text, Box::new(self.parse_expr()?))))
        } else {
            Ok(key_token.to_cst_expr(CstData::Presence(key_text)))
        }
    }

    fn parse_collection(&mut self) -> Result<CstExpression, ParserError> {
        let lparen = self.current().ok_or(ParserError::eoi(&self))?;

        if lparen.kind() != Kind::LParen {
            return Err(ParserError::expected_l_paren(&lparen))
        }
        self.advance();

        let mut collection = vec![];
        while self.eat(Kind::RParen).is_none() {
            collection.push(self.parse_expr()?)
        }

        Ok(lparen.to_cst_expr(CstData::Collection(collection)))
    }

    fn parse_expr(&mut self) -> Result<CstExpression, ParserError> {
        let current = self.current().ok_or(ParserError::eoi(&self))?;

        match current.kind() {
            Kind::LParen => self.parse_collection(),
            Kind::RParen => Err(ParserError::unexpected_r_paren(&current)),
            Kind::Text => self.parse_pair(),
            Kind::Equals => Err(ParserError::unexpected_equals(&current)),
        }
    }

    pub fn parse_tokens(&mut self) -> Result<CstExpression, ParserError> {
        let mut collection = vec![];
        while !self.finished() {
            collection.push(self.parse_expr()?)
        }

        Ok(CstExpression::new(
            CstData::Collection(collection),
            Some(Location {row: 1, column: 1}),
            None
        ).minimized())
    }

    pub fn parse(text: impl AsRef<str>) -> Result<CstExpression, ParserError> {
        Parser::new(text).parse_tokens()
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::CstData::{Collection, Pair, Presence};
    use super::*;

    #[test]
    fn presence() {
        let p = Parser::new("some-key").parse_tokens().unwrap();
        assert_eq!(p, CstExpression::uncommented(Presence(String::from("some-key")), Location::new(1, 1)))
    }

    #[test]
    fn pair() {
        let p = Parser::new("some-key = value").parse_tokens().unwrap();
        assert_eq!(p, CstExpression::uncommented(Pair(
            "some-key".to_string(),
            Box::new(CstExpression::uncommented(
                Presence("value".to_string()),
                Location::new(1, 12)
            ))
        ), Location::new(1, 1)))
    }

    #[test]
    fn paired_collection() {
        let p = Parser::new("some-key = (a b)").parse_tokens().unwrap();
        assert_eq!(p, CstExpression::uncommented(
            Pair("some-key".to_string(), Box::new(CstExpression::uncommented(
                Collection(vec![
                    CstExpression::uncommented(Presence("a".to_string()), Location::new(1, 13)),
                    CstExpression::uncommented(Presence("b".to_string()), Location::new(1, 15))
                ]),
                Location::new(1, 12)
            ))),
            Location::new(1, 1)
        ))
    }

    #[test]
    fn nesting() {
        let p = Parser::new("some-key = (a = b c)").parse_tokens().unwrap();
        assert_eq!(p, CstExpression::uncommented(
            Pair("some-key".to_string(), Box::new(CstExpression::uncommented(
                Collection(vec![
                    CstExpression::uncommented(
                        Pair("a".to_string(), Box::new(CstExpression::uncommented(
                            Presence("b".to_string()),
                            Location::new(1, 17))
                        )),
                        Location::new(1, 13)
                    ),
                    CstExpression::uncommented(Presence("c".to_string()), Location::new(1, 19))
                ]),
                Location::new(1, 12)))),
            Location::new(1, 1)
        ))
    }

    #[test]
    fn collection() {
        let p = Parser::new("(a b)").parse_tokens().unwrap();
        assert_eq!(p, CstExpression::uncommented(Collection(vec![
            CstExpression::uncommented(Presence(String::from("a")), Location::new(1, 2)),
            CstExpression::uncommented(Presence(String::from("b")), Location::new(1, 4)),
        ]), Location::new(1, 1)))
    }

    #[test]
    fn parse_the_thing() {
        let text = r"some-key = value
nested-key = (
    one = 1
    # a comment goes here
    two = 2 # or here
)
escaped-characters = (
    \(
    \)
    \=
    \\
)";
        Parser::new(text).parse_tokens().unwrap();
    }
}