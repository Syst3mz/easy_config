use std::fmt::Display;
use crate::lexical_range::LexicalSpan;
use crate::parser::parser_error::ParserError;

#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum Kind {
    LParen,
    RParen,
    Text,
    Number,
    Equals,
    Eoi
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", match self {
            Kind::LParen => "Left Parenthesis",
            Kind::RParen => "Right Parenthesis",
            Kind::Text => "Text",
            Kind::Number => "Number",
            Kind::Equals => "Equals",
            Kind::Eoi => "End of Input"
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Token {
    kind: Kind,
    span: LexicalSpan,
    lexeme: String
}

fn valid_identifier_first_letter(letter: char) -> bool {
    letter.is_alphabetic() || letter == '_'
}

impl Token {
    pub fn new(kind: Kind, start: usize, lexeme: impl AsRef<str>) -> Self {
        let lexeme = lexeme.as_ref().to_owned();
        Self { kind, span: LexicalSpan::new(start, start + lexeme.len()), lexeme }
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }
    pub fn span(&self) -> LexicalSpan {
        self.span
    }

    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }

    pub fn invalid_identifier_char_index(&self) -> Option<usize> {
        if self.kind != Kind::Text {
            return Some(0)
        }

        let mut chars = self.lexeme.chars().enumerate();
        if let Some((index, c)) = chars.next() {
            if !valid_identifier_first_letter(c) {
                return Some(index)
            }
        }

        for (index, c) in chars {
            if !(c.is_alphanumeric() || c == '_') {
                return Some(index)
            }
        }

        None
    }

    pub fn new_eoi(at: usize) -> Self {
        Self {
            kind: Kind::Eoi,
            span: LexicalSpan::new(at, at + 1),
            lexeme: "End Of Input".to_string(),
        }
    }

    pub fn eoi_check(self, source_text: impl AsRef<str>) -> Result<Self, ParserError> {
        if self.kind == Kind::Eoi {
            Err(ParserError::end_of_input(source_text))
        } else {
            Ok(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_identifier() {
       assert!(Token::new(Kind::Text, 0, "some_name").invalid_identifier_char_index().is_none());
    }

    #[test]
    fn invalid_identifier() {
        assert_eq!(Token::new(Kind::Text, 0, "some-name").invalid_identifier_char_index(), Some(4));
    }
}