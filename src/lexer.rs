use std::cmp::Ordering;

use crate::token::Token;

pub struct Lexer<'a> {
    input: &'a [u8],
    position: usize,
    read_position: usize,
    ch: Option<u8>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer {
            input: input.as_bytes(),
            position: 0,
            read_position: 0,
            ch: None,
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        self.ch = match self.read_position.cmp(&self.input.len()) {
            Ordering::Greater => None,
            Ordering::Equal => Some(0),
            Ordering::Less => Some(self.input[self.read_position]),
        };

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn read_identifier(&mut self) -> &str {
        let start_position = self.position;
        while self.ch.is_some_and(is_letter) {
            self.read_char();
        }

        std::str::from_utf8(&self.input[start_position..self.position]).unwrap()
    }

    fn read_number(&mut self) -> &str {
        let start_position = self.position;
        while self.ch.is_some_and(|ch| ch.is_ascii_digit()) {
            self.read_char();
        }

        std::str::from_utf8(&self.input[start_position..self.position]).unwrap()
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_some_and(|ch| (ch as char).is_whitespace()) {
            self.read_char();
        }
    }

    fn peek_char(&mut self) -> Option<u8> {
        if self.read_position >= self.input.len() {
            None
        } else {
            Some(self.input[self.read_position])
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let ch = match self.ch {
            Some(ch) => ch,
            None => return None,
        };

        let token = match ch {
            b'=' => {
                if self.peek_char() == Some(b'=') {
                    self.read_char();
                    Token::Eq
                } else {
                    Token::Assign
                }
            }
            b';' => Token::Semicolon,
            b'(' => Token::Lparen,
            b')' => Token::Rparen,
            b',' => Token::Comma,
            b'+' => Token::Plus,
            b'-' => Token::Minus,
            b'{' => Token::Lsquigly,
            b'}' => Token::Rsquigly,
            b'!' => {
                if self.peek_char() == Some(b'=') {
                    self.read_char();
                    Token::NotEq
                } else {
                    Token::Bang
                }
            }
            b'/' => Token::Slash,
            b'*' => Token::Asterisk,
            b'<' => Token::Lt,
            b'>' => Token::Gt,
            b'\0' => Token::Eof,
            _ => {
                if is_letter(ch) {
                    let token = Token::lookup_ident(self.read_identifier());
                    // Exit early, because read_char() is called in the read_identifier() function.
                    return Some(token);
                } else if ch.is_ascii_digit() {
                    let token = Token::Int(self.read_number().to_owned());
                    // Exit early, because read_char() is called in the read_number() function.
                    return Some(token);
                } else {
                    Token::Illegal(ch)
                }
            }
        };

        self.read_char();
        Some(token)
    }
}

fn is_letter(ch: u8) -> bool {
    (ch as char).is_alphabetic() || ch == b'_'
}

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::token::Token;

    #[test]
    fn test_next_token() {
        let input = r#"let five = 5;
let ten = 10;

let add = fn(x, y) {
    x + y;
};

let result = add(five, ten);
!-/*5;
5 < 10 > 5;

if (5 < 10) {
    return true;
} else {
    return false;
}

10 == 10;
10 != 9;
"#;

        let expected_values = vec![
            Token::Let,
            Token::Ident("five".to_string()),
            Token::Assign,
            Token::Int("5".to_string()),
            Token::Semicolon,
            Token::Let,
            Token::Ident("ten".to_string()),
            Token::Assign,
            Token::Int("10".to_string()),
            Token::Semicolon,
            Token::Let,
            Token::Ident("add".to_string()),
            Token::Assign,
            Token::Function,
            Token::Lparen,
            Token::Ident("x".to_string()),
            Token::Comma,
            Token::Ident("y".to_string()),
            Token::Rparen,
            Token::Lsquigly,
            Token::Ident("x".to_string()),
            Token::Plus,
            Token::Ident("y".to_string()),
            Token::Semicolon,
            Token::Rsquigly,
            Token::Semicolon,
            Token::Let,
            Token::Ident("result".to_string()),
            Token::Assign,
            Token::Ident("add".to_string()),
            Token::Lparen,
            Token::Ident("five".to_string()),
            Token::Comma,
            Token::Ident("ten".to_string()),
            Token::Rparen,
            Token::Semicolon,
            Token::Bang,
            Token::Minus,
            Token::Slash,
            Token::Asterisk,
            Token::Int("5".to_string()),
            Token::Semicolon,
            Token::Int("5".to_string()),
            Token::Lt,
            Token::Int("10".to_string()),
            Token::Gt,
            Token::Int("5".to_string()),
            Token::Semicolon,
            Token::If,
            Token::Lparen,
            Token::Int("5".to_string()),
            Token::Lt,
            Token::Int("10".to_string()),
            Token::Rparen,
            Token::Lsquigly,
            Token::Return,
            Token::True,
            Token::Semicolon,
            Token::Rsquigly,
            Token::Else,
            Token::Lsquigly,
            Token::Return,
            Token::False,
            Token::Semicolon,
            Token::Rsquigly,
            Token::Int("10".to_string()),
            Token::Eq,
            Token::Int("10".to_string()),
            Token::Semicolon,
            Token::Int("10".to_string()),
            Token::NotEq,
            Token::Int("9".to_string()),
            Token::Semicolon,
            Token::Eof,
        ];

        let mut lexer = Lexer::new(input);

        for (index, expected) in expected_values.iter().enumerate() {
            let token = lexer.next();
            assert_eq!(
                token.as_ref(),
                Some(expected),
                "tests[{}] - token wrong. expected={:?}, got={:?}",
                index,
                expected,
                token
            );
        }
    }
}
