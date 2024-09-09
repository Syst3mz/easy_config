use std::iter::Peekable;
use std::str::Chars;
use crate::lexer::eat_and_expect::EatAndExpect;
use crate::lexer::token::Token;

pub mod token;
mod eat_and_expect;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    cursor_row: usize,
    cursor_column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Lexer<'a> {
        Self {
            chars: text.chars().peekable(),
            cursor_row: 1,
            cursor_column: 0,
        }
    }
    /*

    fn handle_escapes(&mut self) {

        let next_char = if let Some(next_char) = self.chars.peek() {
            *next_char
        } else { return };

        dbg!(&next_char);

        let escaped = match next_char {
            '(' => '(',
            ')' => ')',
            '=' => '=',
            '\\' => '\\',
            _ => {return;}
        };

        dbg!("here");
        self.text_token_lexeme.push(escaped);
        dbg!(&self.text_token_lexeme);
        self.chars.next();
        self.cursor_column += 1;
    }

    fn consume_text(&mut self, starting_char: char) -> Option<Token> {
        self.text_token_lexeme.push(starting_char);

        while let Some(next_char) = self.chars.peek() {
           let next_char = *next_char;
            dbg!(next_char);
            match next_char {
                '\\' => {
                    // advance the iterator consuming the \
                    dbg!("here");
                    self.chars.next();
                    self.cursor_column += 1;
                    self.handle_escapes();
                    continue;
                },
                '(' | ')' | '=' => {
                    return self.close_text_token()
                }
                _ => {}
            }

            if next_char.is_whitespace() {
                break;
            }

            self.cursor_column += 1;
            self.text_token_lexeme.push(self.chars.next().unwrap());
        }

        return self.close_text_token()
    }
    fn consume_char(&mut self, char: char) -> Option<Token> {
        if char.is_whitespace() {
            return None;
        }

        let token = match char {
            '(' => Some(self.tokenize(token::Kind::LParen, "(")),
            ')' => Some(self.tokenize(token::Kind::RParen, ")")),
            '=' => Some(self.tokenize(token::Kind::Equals, "=")),
            _ => self.consume_text(char)
        };

        self.cursor_column += 1;
        token
    }

    */

    fn eat_comment(&mut self) {
        while let Some(char) = self.chars.next() {
            if char == '\n' {
                self.new_line();
                return;
            }
        }
    }

    fn tokenize(&self, kind: token::Kind, lexeme: impl AsRef<str>) -> Token {
        Token::new(
            kind,
            lexeme.as_ref().to_string(),
            self.cursor_row,
            self.cursor_column,
        )
    }

    fn consume_single_char(&mut self, to_consume: char) -> Option<Token> {
        match to_consume {
            '(' => Some(self.tokenize(token::Kind::LParen, "(")),
            ')' => Some(self.tokenize(token::Kind::RParen, ")")),
            '=' => Some(self.tokenize(token::Kind::Equals, "=")),
            _ => None
        }
    }

    fn is_terminal(char: char) -> bool {
        match char {
            '(' | ')' | '=' | '\\' => true,
            _ => false
        }
    }

    fn new_line(&mut self) {
        self.cursor_row += 1;
        self.cursor_column = 0;
    }

    fn deal_with_escapes(maybe_slash: char, maybe_escape: char) -> Option<String> {
        if maybe_slash != '\\' {
            return None;
        }

        if  Self::is_terminal(maybe_escape) {
            Some(String::from(maybe_escape))
        } else {
            Some(String::from('\\'))
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(mut char) = self.chars.next() {
            // if we see two unbroken slashes that means we are in a comment and should ignore all
            // input until a newline.
            if char == '/' && self.chars.eat(|x| *x == '/').is_some() {
                self.eat_comment();
                // we will start again after the newline.
                continue;
            }

            if char == '\n' {
                self.new_line();
                continue;
            }

            self.cursor_column += 1;
            if char.is_whitespace() {
                continue;
            }

            let maybe_single_character_token = self.consume_single_char(char);
            if maybe_single_character_token.is_some() {
                return maybe_single_character_token;
            }

            // not whitespace or a single char token
            let mut lexeme = String::new();
            let mut pair = (char, char);

            while let Some(peeked_char) = self.chars.peek() {
                let peeked_char = *peeked_char;
                pair = (char, peeked_char);

                println!("{:?}", pair);
                println!("{:?}", Self::deal_with_escapes(pair.0, pair.1));



                char = peeked_char;
                self.next();
            }


            let lexeme_len = lexeme.len();
            return Some(Token::new(
                token::Kind::Text,
                lexeme,
                self.cursor_row,
                1 + self.cursor_column - lexeme_len
            ));
        }

        return None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lparen() {
        let text = "(";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::LParen, "(", 1, 1)))
    }

    #[test]
    fn rparen() {
        let text = ")";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::RParen, ")", 1, 1)))
    }

    #[test]
    fn lparen_rparen() {
        let text = "()";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::LParen, "(", 1, 1)));
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::RParen, ")", 1, 2)));
    }

    #[test]
    fn lparen_rparen_inline_comment() {
        let text = "(//comment in line\n)";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::LParen, "(", 1, 1)));
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::RParen, ")", 2, 1)));
    }
    #[test]
    fn lparen_rparen_comment() {
        let text = "//comment not inline\n()";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::LParen, "(", 2, 1)));
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::RParen, ")", 2, 2)));
    }

    #[test]
    fn lparen_rparen_terminating_comment() {
        let text = "()//comment not inline";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::LParen, "(", 1, 1)));
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::RParen, ")", 1, 2)));
    }

    #[test]
    fn rocket() {
        let text = "rocket";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::Text, "rocket", 1, 1)));
    }

    #[test]
    fn lparen_rocket_rparen() {
        let text = "(rocket)";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::LParen, "(", 1, 1)));
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::Text, "rocket", 1, 2)));
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::RParen, ")", 1, 8)));
    }

    #[test]
    fn escaped_l_paren() {
        let text = r"\(";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::Text, "(", 1, 1)));
    }

    #[test]
    fn escaped_lparen_b() {
        let text = r"\(b";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::Text, "(b", 1, 1)));
    }

    #[test]
    fn b_escaped_lparen() {
        let text = r"b\(";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::Text, "b(", 1, 2)));
    }

    #[test]
    fn a_escaped_lparen_b() {
        let text = r"a\(b";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::Text, "a(b", 1, 2)));
    }
}