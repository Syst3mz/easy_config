use crate::expression::Expression;
use crate::expression::Expression::Pair;
use crate::lexer::{Lexer, token};
use crate::lexer::token::{Kind, Token};
use crate::lexer::token::Kind::Text;
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

    fn parse_pair(&mut self) -> Result<Expression, ParserError> {
        let current_token = self.current().ok_or(ParserError::eoi(&self))?;
        if current_token.kind() != Text {
            return Err(ParserError::expected_text(&current_token))
        }
        self.advance();

        let token_text = current_token.lexeme().to_string();

        if self.eat(Kind::Equals).is_some() {

            Ok(Pair(token_text, Box::new(self.parse_expr()?)))
        } else {
            Ok(Expression::Presence(token_text))
        }
    }

    fn parse_collection(&mut self) -> Result<Expression, ParserError> {
        let lparen = self.current().ok_or(ParserError::eoi(&self))?;

        if lparen.kind() != Kind::LParen {
            return Err(ParserError::expected_l_paren(&lparen))
        }
        self.advance();

        let mut collection = vec![];
        while self.eat(Kind::RParen).is_none() {
            collection.push(self.parse_expr()?)
        }

        Ok(Expression::Collection(collection))
    }

    fn parse_expr(&mut self) -> Result<Expression, ParserError> {
        let current = self.current().ok_or(ParserError::eoi(&self))?;

        match current.kind() {
            Kind::LParen => self.parse_collection(),
            Kind::RParen => Err(ParserError::unexpected_r_paren(&current)),
            Kind::Text => self.parse_pair(),
            Kind::Equals => Err(ParserError::unexpected_equals(&current)),
        }
    }

    pub fn parse(&mut self) -> Result<Expression, ParserError> {
        let mut collection = vec![];
        while !self.finished() {
            collection.push(self.parse_expr()?)
        }

        Ok(Expression::Collection(collection).minimized())
    }
}

#[cfg(test)]
mod tests {
    use crate::expression::Expression::{Collection, Presence};
    use super::*;

    #[test]
    fn presence() {
        let p = Parser::new("some-key").parse().unwrap();
        assert_eq!(p, Presence(String::from("some-key")))
    }

    #[test]
    fn pair() {
        let p = Parser::new("some-key = value").parse().unwrap();
        assert_eq!(p, Pair(String::from("some-key"), Box::new(Presence(String::from("value")))))
    }

    #[test]
    fn paired_collection() {
        let p = Parser::new("some-key = (a b)").parse().unwrap();
        assert_eq!(p, Pair(String::from("some-key"), Box::new(Collection(vec![
            Presence(String::from("a")),
            Presence(String::from("b"))
        ]))))
    }

    #[test]
    fn nesting() {
        let p = Parser::new("some-key = (a = b c)").parse().unwrap();
        assert_eq!(p, Pair(String::from("some-key"), Box::new(Collection(vec![
            Pair(String::from("a"), Box::new(Presence(String::from("b")))),
            Presence(String::from("c"))
        ]))))
    }

    #[test]
    fn collection() {
        let p = Parser::new("(a b)").parse().unwrap();
        assert_eq!(p, Collection(vec![Presence(String::from("a")), Presence(String::from("b"))]))
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
        Parser::new(text).parse().unwrap();
    }
}