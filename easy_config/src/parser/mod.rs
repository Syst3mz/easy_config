use itertools::Itertools;
use crate::config_error::Contextualize;
use crate::expression::{Atom, Expression};
use crate::lexer::{token, Lexer};
use crate::lexer::token::{Kind, Token};
use crate::lexical_span::LexicalSpan;
use crate::parser::parser_error::{end_of_input, ParserError};

pub mod parser_error;
pub struct FinishedParser {
    expression: Expression,
    errors: Vec<ParserError>
}

impl From<Parser> for FinishedParser {
    fn from(parser: Parser) -> Self {
        let span = LexicalSpan::new(
            parser.expressions.first().unwrap().span().start(),
            parser.expressions.last().unwrap().span().end()
        );
        Self {
            expression: Expression::list(parser.expressions).with_span(span),
            errors: parser.errors,
        }
    }
}
impl FinishedParser {
    pub fn errors(&self) -> &Vec<ParserError> {
        &self.errors
    }

    pub fn unwrap(self) -> Expression {
        if !self.errors.is_empty() {
            let panic_text= self.errors.iter()
                .map(|x| x.to_string())
                .join("\n");
            panic!("{}", panic_text);
        }

        self.expression
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    current_index: usize,
    expressions: Vec<Expression>,
    errors: Vec<ParserError>,
    source: String
}

/*
enum -> TEXT list
atom -> NUMBER
      | TEXT

presence -> atom
bind -> TEXT "=" expression
list -> "(" expression* ")"
expression -> presence
            | bind
            | enum
            | list
*/
type Tk = token::Kind;
type Ek = parser_error::Kind;
impl Parser {
    pub fn new(text: impl AsRef<str>) -> Parser {
        let text = text.as_ref();
        Self {
            tokens: Lexer::new(text).collect(),
            current_index: 0,
            expressions: vec![],
            errors: vec![],
            source: text.to_string()
        }
    }

    fn advance(&mut self) {
        self.current_index += 1;
    }

    fn finished(&self) -> bool {
        self.current_index == self.tokens.len()
    }

    fn get(&self, at: usize) -> Token {
        self.tokens.get(at).unwrap_or(&Token::new_eoi(at)).clone()
    }

    fn current(&self) -> Token {
        self.get(self.current_index)
    }

    fn next(&mut self) -> Token {
        let current_index = self.current_index;
        self.advance();
        self.get(current_index)
    }
    fn eat(&mut self, kind: Tk) -> Result<Token, Token> {
        let t = self.expect(kind);
        if t.is_ok() {
            self.advance();
        }

        t
    }
    fn expect(&self, kind: Tk) -> Result<Token, Token> {
        let current = self.current();
        if current.kind() == kind {
            Ok(current)
        } else {
            Err(current)
        }
    }
    fn unexpected_token_error(&self, offender: Token, expected: &'static [Tk]) -> ParserError {
        let span = offender.span();
        ParserError::on_span(
            Ek::UnexpectedToken(offender, expected),
            span,
            &self.source
        )
    }
    fn parse_atom(&mut self) -> Result<Token, ParserError> {
        let token = self.next().eoi_check(&self.source)?;

        if token.kind() == Tk::Text {
            return Ok(token)
        }

        if token.kind() == Tk::Number {
            return Ok(token)
        }

        Err(self.unexpected_token_error(token, &[Tk::Text, Tk::Number]))
    }

    fn parse_binding(&mut self, identifier: Token) -> Result<Expression, ParserError> {
        if let Some(errant_index) = identifier.invalid_identifier_char_index() {
            let errant_index = identifier.span().start() + errant_index;
            return Err(ParserError::on_span(Ek::InvalidIdentifier(identifier), LexicalSpan::new(errant_index, errant_index + 1), &self.source))
        }

        if let Err(token) = self.eat(Tk::Equals) {
            let token = token.eoi_check(&self.source)?;
            return Err(self.unexpected_token_error(token, &[Tk::Equals]))
        }

        let value = self
            .parse_expression()
            .contextualize(format!(
                "Failed to parse the value of the binding '{}'.",
                identifier.lexeme())
            )?;

        if let Ok(l_paren) = self.eat(Tk::LParen) {
            let mut list = self.parse_list(l_paren)?;
            list.prepend_into_list(value);

            let span = identifier.span().combine(list.span());
            return Ok(Expression::binding(identifier.lexeme(), list).with_span(span));
        }

        let span = identifier.span().combine(value.span());
        Ok(Expression::binding(identifier.lexeme(), value).with_span(span))
    }

    fn parse_list(&mut self, l_paren: Token) -> Result<Expression, ParserError> {
        let mut elements = vec![];

        loop {
            if let Ok(r_paren) = self.eat(Tk::RParen) {
                return Ok(
                    Expression::list(elements)
                    .with_span(l_paren.span().combine(r_paren.span()))
                )
            }

            elements.push(self.parse_expression()?);
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
        if self.finished() {
            return Err(end_of_input(&self.source).contextualize("Tried to parse an expression at the end of the input."))
        }
        if let Ok(l_paren) = self.eat(Tk::LParen) {
            return self.parse_list(l_paren).contextualize("Tried to parse a list as an expression.");
        }

        let name = self.parse_atom()
            .map_err(|x| x.contextualize("Expected an atom that was either a presence or a binding."))?;

        if self.expect(Tk::Equals).is_ok() {
            return self.parse_binding(name)
        }

        let atom = match name.kind() {
            Kind::Number => Atom::Number(name.lexeme().to_string()),
            Kind::Text => Atom::Text(name.lexeme().to_string()),
            _ => return Err(self.unexpected_token_error(name, &[Tk::Text, Tk::Number])),
        };

        Ok(Expression::presence(atom).with_span(name.span()))
    }

    pub fn parse(mut self) -> FinishedParser {
        while !self.finished() {
            match self.parse_expression() {
                Ok(o) => self.expressions.push(o),
                Err(e) => self.errors.push(e),
            }

        }

        FinishedParser::from(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexical_span::LexicalSpan;
    use super::*;

    #[test]
    fn presence() {
        let p = Parser::new("some_key").parse().unwrap();
        assert_eq!(
            p.get_first_expr_or_panic(),
            Expression::presence("some_key").with_span(LexicalSpan::new(0, 8))
        );
    }

    #[test]
    fn binding() {
        let p = Parser::new("some_key = value").parse().unwrap();
        assert_eq!(
            p.get_first_expr_or_panic(),
            Expression::binding(
                "some_key",
                Expression::presence("value").with_span(LexicalSpan::new(11, 16))
            )
                .with_span(LexicalSpan::new(0, 16))
        );
    }

    #[test]
    fn bound_collection() {
        let p = Parser::new("some_key = (a b)").parse().unwrap();
        assert_eq!(
            p.get_first_expr_or_panic(),
            Expression::binding(
                "some_key",
                Expression::list(vec![
                    Expression::presence("a").with_span(LexicalSpan::new(12, 13)),
                    Expression::presence("b").with_span(LexicalSpan::new(14, 15)),
                ])
                    .with_span(LexicalSpan::new(11, 16))
            )
                .with_span(LexicalSpan::new(0, 16))
        );
    }

    #[test]
    fn nesting() {
        let p = Parser::new("some_key = (a = b c)").parse().unwrap();
        let inner_binding = Expression::binding(
            "a",
            Expression::presence("b").with_span(LexicalSpan::new(16, 17)),
        )
            .with_span(LexicalSpan::new(12, 17));

        let c_presence = Expression::presence("c").with_span(LexicalSpan::new(18, 19));

        let list = Expression::list(vec![inner_binding, c_presence]).with_span(LexicalSpan::new(11, 20));

        let top_level = Expression::binding("some_key", list).with_span(LexicalSpan::new(0, 20));

        assert_eq!(p.get_first_expr_or_panic(), top_level);
    }

    #[test]
    fn collection() {
        let p = Parser::new("(a b)").parse().unwrap();
        assert_eq!(
            p.get_first_expr_or_panic(),
            Expression::list(vec![
                Expression::presence("a").with_span(LexicalSpan::new(1, 2)),
                Expression::presence("b").with_span(LexicalSpan::new(3, 4)),
            ])
                .with_span(LexicalSpan::new(0, 5))
        );
    }

    #[test]
    fn parse_single_enum() {
        let p = Parser::new("bind = Some(thing)").parse().unwrap();
        let enum_expr = Expression::list(vec![
            Expression::presence("Some").with_span(LexicalSpan::new(7, 11)),
            Expression::presence("thing").with_span(LexicalSpan::new(12, 17)),
        ])
            .with_span(LexicalSpan::new(11, 18));

        let binding = Expression::binding("bind", enum_expr).with_span(LexicalSpan::new(0, 18));

        assert_eq!(p.get_first_expr_or_panic(), binding);
    }

    #[test]
    #[should_panic]
    fn unclosed_list() {
         Parser::new("(a b").parse().unwrap();
    }


    #[test]
    fn parse_the_thing() {
        let text = r"some_key = value
nested_key = (
    one = 1
    # a comment goes here
    two = 2 # or here
)
escaped_characters = (
    \(
    \)
    \=
    \\
)";
        Parser::new(text).parse().unwrap();
    }
}