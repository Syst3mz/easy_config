use logos::Logos;
use crate::error::Error;
use crate::location::Location;
use crate::span::Span;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Logos)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip(r"#[^\n]*", allow_greedy = true))]
pub enum TokenKind {
    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token("=")]
    Equals,

    #[regex(r"(?:[^()=\\\s#]|\\.)+")]
    Text,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub span: Span,
    pub lexeme: &'a str,
}

impl<'a> Token<'a> {
    pub fn locate(&self, source: impl AsRef<str>) -> Location {
        let mut line = 1;
        let mut column = 0;
        for char in source.as_ref().chars().take(self.span.start + 1) {
            if char == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
        }

        Location {
            line,
            column,
        }
    }
}

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, TokenKind>
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: TokenKind::lexer(input),
        }
    }
}

impl <'a> Iterator for Lexer<'a> {
    type Item = Result<Token<'a>, Error>;
    fn next(&mut self) -> Option<Self::Item> {

        let token = self.inner.next()?;
        Some(token.map(|kind| Token {
            kind,
            span: self.inner.span().into(),
            lexeme: self.inner.slice(),
        }).map_err(|_| Error::TokenizationError(self.inner.span().into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(input: &str) -> Vec<Result<TokenKind, Error>> {
        Lexer::new(input).map(|r| r.map(|t| t.kind)).collect()
    }

    fn lex_lexemes(input: &str) -> Vec<&str> {
        Lexer::new(input)
            .filter_map(|r| r.ok())
            .map(|t| t.lexeme)
            .collect()
    }

    // --- whitespace skipping ---

    #[test]
    fn skips_spaces() {
        assert_eq!(lex("  (  )  "), vec![Ok(TokenKind::LeftParen), Ok(TokenKind::RightParen)]);
    }

    #[test]
    fn skips_tabs_and_newlines() {
        assert_eq!(lex("\t(\n)"), vec![Ok(TokenKind::LeftParen), Ok(TokenKind::RightParen)]);
    }

    // --- parens ---

    #[test]
    fn lone_left_paren() {
        assert_eq!(lex("("), vec![Ok(TokenKind::LeftParen)]);
    }

    #[test]
    fn lone_right_paren() {
        assert_eq!(lex(")"), vec![Ok(TokenKind::RightParen)]);
    }

    #[test]
    fn nested_parens() {
        assert_eq!(
            lex("(())"),
            vec![
                Ok(TokenKind::LeftParen),
                Ok(TokenKind::LeftParen),
                Ok(TokenKind::RightParen),
                Ok(TokenKind::RightParen),
            ]
        );
    }

    // --- text ---

    #[test]
    fn simple_word() {
        assert_eq!(lex("hello"), vec![Ok(TokenKind::Text)]);
        assert_eq!(lex_lexemes("hello"), vec!["hello"]);
    }

    #[test]
    fn multiple_words_separated_by_spaces() {
        assert_eq!(lex("foo bar"), vec![Ok(TokenKind::Text), Ok(TokenKind::Text)]);
        assert_eq!(lex_lexemes("foo bar"), vec!["foo", "bar"]);
    }

    #[test]
    fn text_with_equals() {
        assert_eq!(lex("x=1"), vec![Ok(TokenKind::Text), Ok(TokenKind::Equals), Ok(TokenKind::Text)]);
    }

    // --- escape sequences ---

    #[test]
    fn escaped_left_paren_is_text() {
        assert_eq!(lex(r"\("), vec![Ok(TokenKind::Text)]);
        assert_eq!(lex_lexemes(r"\("), vec![r"\("]);
    }

    #[test]
    fn escaped_right_paren_is_text() {
        assert_eq!(lex(r"\)"), vec![Ok(TokenKind::Text)]);
        assert_eq!(lex_lexemes(r"\)"), vec![r"\)"]);
    }

    #[test]
    fn escaped_equals_is_text() {
        assert_eq!(lex(r"\="), vec![Ok(TokenKind::Text)]);
        assert_eq!(lex_lexemes(r"\="), vec![r"\="]);
    }

    #[test]
    fn escaped_backslash_is_text() {
        assert_eq!(lex(r"\\"), vec![Ok(TokenKind::Text)]);
        assert_eq!(lex_lexemes(r"\\"), vec![r"\\"]);
    }

    #[test]
    fn escape_in_the_middle_of_word() {
        assert_eq!(lex(r"hello\(world"), vec![Ok(TokenKind::Text)]);
        assert_eq!(lex_lexemes(r"hello\(world"), vec![r"hello\(world"]);
    }

    #[test]
    fn escape_does_not_bleed_into_next_token() {
        assert_eq!(
            lex(r"foo\((bar)"),
            vec![
                Ok(TokenKind::Text),      // foo\(
                Ok(TokenKind::LeftParen), // (
                Ok(TokenKind::Text),      // bar
                Ok(TokenKind::RightParen),
            ]
        );
        assert_eq!(lex_lexemes(r"foo\((bar)"), vec![r"foo\(", "(", "bar", ")"]);
    }

    // --- realistic inputs ---

    #[test]
    fn simple_list() {
        assert_eq!(
            lex("(a b c)"),
            vec![
                Ok(TokenKind::LeftParen),
                Ok(TokenKind::Text),
                Ok(TokenKind::Text),
                Ok(TokenKind::Text),
                Ok(TokenKind::RightParen),
            ]
        );
        // lex_lexemes includes all token lexemes, parens included
        assert_eq!(lex_lexemes("(a b c)"), vec!["(", "a", "b", "c", ")"]);
    }

    #[test]
    fn struct_like() {
        let kinds = lex("(x=1 y=2)");
        assert_eq!(kinds[0], Ok(TokenKind::LeftParen));
        assert_eq!(kinds[1], Ok(TokenKind::Text));   // "x"
        assert_eq!(kinds[2], Ok(TokenKind::Equals));  // "="
        assert_eq!(kinds[3], Ok(TokenKind::Text));   // "1"
        assert_eq!(kinds[4], Ok(TokenKind::Text));   // "y"
        assert_eq!(kinds[5], Ok(TokenKind::Equals));  // "="
        assert_eq!(kinds[6], Ok(TokenKind::Text));   // "2"
        assert_eq!(kinds[7], Ok(TokenKind::RightParen));
    }

    #[test]
    fn nested_struct() {
        assert_eq!(
            lex_lexemes("(Position (x=1 y=2))"),
            vec!["(", "Position", "(", "x", "=", "1", "y", "=", "2", ")", ")"]
        );
    }

    #[test]
    fn empty_input_produces_no_tokens() {
        assert_eq!(lex(""), vec![]);
    }

    #[test]
    fn only_whitespace_produces_no_tokens() {
        assert_eq!(lex("   \t\n  "), vec![]);
    }
}