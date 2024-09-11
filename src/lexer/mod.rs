use crate::lexer::str_extensions::StrExtensions;
use crate::lexer::token::Token;


pub mod token;
mod str_extensions;

pub struct Lexer<'a> {
    text: &'a str,
    cursor_row: usize,
    cursor_column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(text: &'a str) -> Lexer<'a> {
        Self {
            text,
            cursor_row: 1,
            cursor_column: 1,
        }
    }
    fn new_line(&mut self) {
        self.cursor_row += 1;
        self.cursor_column = 1;
    }

    fn place_token(&self, kind: token::Kind, lexeme: impl AsRef<str>) -> Token {
        Token::new(
            kind,
            lexeme.as_ref().to_string(),
            self.cursor_row,
            self.cursor_column,
        )
    }

    fn consume_single_char(&mut self, to_consume: char) -> Option<Token> {
        match to_consume {
            '(' => Some(self.place_token(token::Kind::LParen, "(")),
            ')' => Some(self.place_token(token::Kind::RParen, ")")),
            '=' => Some(self.place_token(token::Kind::Equals, "=")),
            _ => None
        }
    }

    fn eat_comment(&mut self) {
        self.skip();
        // eat everything that is not a new line.
        self.take_while(|x| x != '\n');
    }

    fn skip(&mut self) {
        self.text.skip(1);
        self.cursor_column += 1;
    }

    fn take_while(&mut self, accept: impl FnMut(char) -> bool) -> &str {
        self.text.take_while(accept)
    }

    fn first(&self) -> Option<char> {
        self.text.first()
    }

    fn needs_escaping(char: char) -> bool {
        matches!(char, '(' | ')' | '=' | '#' )
    }
    fn token_boundary(char: char) -> bool {
        char.is_whitespace() || Self::needs_escaping(char)
    }
    fn consume_text_token(&mut self) -> Token {
        let mut text = String::new();
        let start_of_token = self.cursor_column;

        loop {
            let segment = self.take_while(|x| !Self::token_boundary(x)).to_string();
            self.cursor_column += segment.len();
            text.push_str(&segment);


            let last_char_is_backslash = segment.chars().last()
                .map(|x| x == '\\')
                .unwrap_or(false);

            let next_char_needs_escaping = self.first()
                .filter(|x| Self::needs_escaping(*x));


            if last_char_is_backslash && next_char_needs_escaping.is_some() {
                let escaped_char = next_char_needs_escaping.unwrap();
                self.skip();
                text.replace_range(text.len()-1..text.len(), escaped_char.to_string().as_str());
            } else {
                break;
            }

        }

        let token = Token::new(token::Kind::Text, text, self.cursor_row, start_of_token);

        token
    }

    fn next_token(&mut self) -> Option<Token> {
        while let Some(c) = self.first() {
            if c == '\n' {
                self.skip();
                self.new_line();
                continue;
            }

            if c.is_whitespace() {
                self.skip();
                continue;
            }

            if c == '#' {
                self.eat_comment();
                continue;
            }

            if let Some(token) = self.consume_single_char(c) {
                self.skip();
                return Some(token)
            }

            return Some(self.consume_text_token())
        }
        None
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eat_rocket() {
        let mut x = "rocket beans";
        assert_eq!(x.take_while(|x| !Lexer::token_boundary(x)), "rocket");
        assert_eq!(x, " beans")
    }

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
        let text = "(#comment in line\n)";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::LParen, "(", 1, 1)));
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::RParen, ")", 2, 1)));
    }
    #[test]
    fn lparen_rparen_comment() {
        let text = "#comment not inline\n()";
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
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::Text, "b(", 1, 1)));
    }

    #[test]
    fn a_escaped_lparen_b() {
        let text = r"a\(b";
        let mut lexer = Lexer::new(text);
        assert_eq!(lexer.next(), Some(Token::new(token::Kind::Text, "a(b", 1, 1)));
    }

    #[test]
    fn newline_inside() {
        let text = "a\nb";
        let mut tokens = Lexer::new(text);
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Text, "a", 1, 1));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Text, "b", 2, 1));
    }

    #[test]
    fn no_escape() {
        let text = r"a\b";
        let mut tokens = Lexer::new(text);
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Text, r"a\b", 1, 1));
    }

    #[test]
    fn long_test_1() {
        let text = r"symbols = (\( \= \#) letters = (a b c)";
        let mut tokens = Lexer::new(text);
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Text, "symbols", 1, 1));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Equals, "=", 1, 9));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::LParen, "(", 1, 11));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Text, "(", 1, 12));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Text, "=", 1, 15));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Text, "#", 1, 18));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::RParen, ")", 1, 20));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Text, "letters", 1, 22));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Equals, "=", 1, 30));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::LParen, "(", 1, 32));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Text, "a", 1, 33));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Text, "b", 1, 35));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::Text, "c", 1, 37));
        assert_eq!(tokens.next().unwrap(), Token::new(token::Kind::RParen, ")", 1, 38));
    }
}