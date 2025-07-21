use itertools::Itertools;
use crate::expression::{Atom, Expression, ExpressionData};
use crate::expression::ExpressionData::Presence;
use crate::lexer::{token, Lexer};
use crate::lexer::token::{Kind, Token};
use crate::lexical_range::LexicalSpan;
use crate::parser::parser_error::{Contextualize, ParserError};

pub mod parser_error;
pub struct FinishedParser {
    expressions: Vec<Expression>,
    errors: Vec<ParserError>
}

impl From<Parser> for FinishedParser {
    fn from(parser: Parser) -> Self {
        Self {
            expressions: parser.expressions,
            errors: parser.errors,
        }
    }
}
impl FinishedParser {
    pub fn expressions(&self) -> &[Expression] {
        &self.expressions
    }
    pub fn errors(&self) -> &[ParserError] {
        &self.errors
    }

    pub fn unwrap(self) -> Vec<Expression> {
        if !self.errors.is_empty() {
            let panic_text= self.errors.iter()
                .map(|x| x.to_string())
                .join("\n");
            panic!("{}", panic_text);
        }

        self.expressions
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
        let span = LexicalSpan::new(identifier.span().start(), value.lexical_range.unwrap().end());
        Ok(Expression::uncommented(
            ExpressionData::Binding(identifier.lexeme().to_string(), Box::new(value)),
            span
        ))
    }

    fn parse_list(&mut self, l_paren: Token) -> Result<Expression, ParserError> {
        let mut elements = vec![];

        loop {
            if let Ok(r_paren) = self.eat(Tk::RParen) {
                return Ok(Expression::uncommented(
                    ExpressionData::List(elements),
                    LexicalSpan::new(l_paren.span().start(), r_paren.span().end())
                ))
            }

            elements.push(self.parse_expression()?);
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, ParserError> {
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

        Ok(Expression::uncommented(Presence(atom), name.span()))
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
    use crate::expression::ExpressionData::{List, Binding, Presence};
    use crate::lexical_range::LexicalSpan;
    use super::*;

    #[test]
    fn presence() {
        let p = Parser::new("some-key").parse().unwrap();
        assert_eq!(p[0], Expression::uncommented(
            Presence("some-key".into()),
            LexicalSpan::new(0, 8)
        ))
    }

    #[test]
    fn binding() {
        let p = Parser::new("some_key = value").parse().unwrap();
        assert_eq!(p[0], Expression::uncommented(Binding(
            "some_key".to_string(),
            Box::new(Expression::uncommented(
                Presence("value".into()),
                LexicalSpan::new(11, 16)
            ))
        ), LexicalSpan::new(0, 16)));
    }

    #[test]
    fn bound_collection() {
        let p = Parser::new("some_key = (a b)").parse().unwrap();
        assert_eq!(p[0], Expression::uncommented(
            Binding("some_key".to_string(), Box::new(Expression::uncommented(
                List(vec![
                    Expression::uncommented(Presence("a".into()), LexicalSpan::new(12, 13)),
                    Expression::uncommented(Presence("b".into()), LexicalSpan::new(14, 15))
                ]),
                LexicalSpan::new(11, 16)
            ))),
            LexicalSpan::new(0, 16)
        ))
    }

    #[test]
    fn nesting() {
        let p = Parser::new("some_key = (a = b c)").parse().unwrap();
        let list = Expression::uncommented(List(vec![
            Expression::uncommented(
                Binding("a".into(), Box::new(Expression::uncommented(
                    Presence("b".into()),
                    LexicalSpan::new(16, 17))
                )),
                LexicalSpan::new(12, 17)
            ),
            Expression::uncommented(Presence("c".into()), LexicalSpan::new(18, 19))
        ]),
           LexicalSpan::new(11, 20)
        );
        let binding =  Expression::uncommented(
            Binding("some_key".into(), Box::new(list)),
            LexicalSpan::new(0, 20)
        );

        assert_eq!(p[0], binding)
    }

    #[test]
    fn collection() {
        let p = Parser::new("(a b)").parse().unwrap();
        assert_eq!(p[0], Expression::uncommented(List(vec![
            Expression::uncommented(Presence("a".into()), LexicalSpan::new(1, 2)),
            Expression::uncommented(Presence("b".into()), LexicalSpan::new(3, 4)),
        ]), LexicalSpan::new(0, 5)))
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