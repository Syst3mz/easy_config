use std::iter::{Enumerate, Peekable};
use std::str::Chars;
use itertools::Itertools;
use crate::lexer::token::{Kind, Token};

pub mod token;
pub const STOPPING_CHARS: [char; 4] = ['(', '=', ')', '#'];
fn is_token_boundary(c: char) -> bool {
    c.is_whitespace() ||
    STOPPING_CHARS.contains(&c)
}

pub(crate) struct Lexer<'a> {
    characters: Peekable<Enumerate<Chars<'a>>>
}

fn potentially_numeric(character: char) -> bool {
    character.is_numeric() ||
    character == 'x'       ||
    character == 'b'       ||
    character == 'e'       ||
    character == '.'
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(input: &'a str) -> Self {
        Self {
            characters: input.chars().enumerate().peekable(),
        }
    }

    fn take_chars_while(&mut self, func: fn(char) -> bool) -> Vec<char> {
        let mut result = Vec::new();

        while let Some((_, c)) = self.characters.peek() {
            if func(*c) {
                result.push(self.characters.next().unwrap().1);
            }
            else {
                break
            }
        }

        result
    }

    fn skip_chars_while(&mut self, func: fn(char) -> bool) {
        while let Some((_, c)) = self.characters.peek() {
            if func(*c) {
                self.characters.next();
            } else {
                break
            }
        }
    }

    fn eat_text(&mut self, first_char: char) -> String {
        let mut text = String::from(first_char);

        loop {
            let ate = self
                .take_chars_while(|x| !is_token_boundary(x))
                .iter()
                .join("");
            text.push_str(&ate);


            assert!(!text.is_empty());
            if text.chars().last().unwrap() != '\\' { break }

            let Some((_, last_char)) = self.characters.next() else { break };
            text.push(last_char);
        }

        text
    }

    fn emit_multi_char_token(&mut self, first_char: char, starting_index: usize) -> Token {
        if first_char.is_numeric() {
            let remainder = self
                .take_chars_while(|c| potentially_numeric(c))
                .iter()
                .join("");
            return Token::new(Kind::Number, starting_index, format!("{}{}", first_char, remainder));
        }

        Token::new(Kind::Text, starting_index, self.eat_text(first_char))
    }

    fn eat_whitespace(&mut self) {
        self.skip_chars_while(|x| x.is_whitespace());
    }

    fn eat_comment(&mut self) {
        if self.characters.peek().map(|(_, x)| *x != '#').unwrap_or(true) {
            return;
        }
        self.skip_chars_while(|x| x != '\n');
        // skip the \n left untouched from above
        self.characters.next();
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.eat_whitespace();
        self.eat_comment();

        // It's not safe to unwrap here, since the loop above could have terminated because peek
        // (and therefore also next) is None.
        let Some((next_index, next_char)) = self.characters.next() else { return None };


        Some(match next_char {
            '(' => Token::new(Kind::LParen, next_index, "("),
            '=' => Token::new(Kind::Equals, next_index, "="),
            ')' => Token::new(Kind::RParen, next_index, ")"),
            c => self.emit_multi_char_token(c, next_index)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eat_rocket() {
        let mut x = Lexer::new("rocket beans");
        assert_eq!(x.take_chars_while(|x| !is_token_boundary(x)).iter().join(""), "rocket")
    }

    #[test]
    fn lparen() {
        let text = "(";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(Kind::LParen, 0, "(")))
    }

    #[test]
    fn rparen() {
        let text = ")";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(Kind::RParen, 0, ")")))
    }

    #[test]
    fn lparen_rparen() {
        let text = "()";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(Kind::LParen, 0, "(")));
        assert_eq!(lexer.next(), Some(Token::new(Kind::RParen, 1, ")")));
    }

    #[test]
    fn lparen_rparen_inline_comment() {
        let text = "(#comment in line\n)";
        let tokens: Vec<Token> = Lexer::new(text).collect();

        let expected = vec![
            Token::new(Kind::LParen, 0, "("),
            Token::new(Kind::RParen, 18, ")")
        ];

        assert_eq!(tokens, expected);
    }
    #[test]
    fn lparen_rparen_comment() {
        let text = "#comment not inline\n()";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(Kind::LParen, 20, "(")));
        assert_eq!(lexer.next(), Some(Token::new(Kind::RParen, 21, ")")));
    }

    #[test]
    fn rocket() {
        let text = "rocket";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(Kind::Text, 0, "rocket")));
    }

    #[test]
    fn lparen_rocket_rparen() {
        let text = "(rocket)";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(Kind::LParen, 0, "(")));
        assert_eq!(lexer.next(), Some(Token::new(Kind::Text, 1, "rocket")));
        assert_eq!(lexer.next(), Some(Token::new(Kind::RParen, 7, ")")));
    }

    #[test]
    fn escaped_l_paren() {
        let text = r"\(";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(Kind::Text, 0, "\\(")));
    }

    #[test]
    fn escaped_lparen_b() {
        let text = r"\(b";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(Kind::Text, 0, "\\(b")));
    }

    #[test]
    fn b_escaped_lparen() {
        let text = r"b\(";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(Kind::Text,  0, "b\\(")));
    }

    #[test]
    fn a_escaped_lparen_b() {
        let text = r"a\(b";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(Kind::Text, 0, "a\\(b")));
    }

    #[test]
    fn newline_inside() {
        let text = "a\nb";
        let mut tokens = Lexer::new(text);
        assert_eq!(tokens.next().unwrap(), Token::new(Kind::Text, 0, "a"));
        assert_eq!(tokens.next().unwrap(), Token::new(Kind::Text, 2, "b"));
    }

    #[test]
    fn long_test_1() {
        let text = r"symbols = (\( \= \#) letters = (a b c)";
        let tokens: Vec<Token> = Lexer::new(text).collect();
        let expected = vec![
            Token::new(Kind::Text, 0, "symbols"),
            Token::new(Kind::Equals, 8, "="),
            Token::new(Kind::LParen, 10, "("),
            Token::new(Kind::Text, 11, "\\("),
            Token::new(Kind::Text, 14, "\\="),
            Token::new(Kind::Text, 17, "\\#"),
            Token::new(Kind::RParen, 19, ")"),
            Token::new(Kind::Text, 21, "letters"),
            Token::new(Kind::Equals, 29, "="),
            Token::new(Kind::LParen, 31, "("),
            Token::new(Kind::Text, 32, "a"),
            Token::new(Kind::Text, 34, "b"),
            Token::new(Kind::Text, 36, "c"),
            Token::new(Kind::RParen, 37, ")"),
        ];

        assert_eq!(tokens, expected);
    }
}