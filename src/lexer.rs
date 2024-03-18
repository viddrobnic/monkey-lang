use crate::token::Token;

pub struct Lexer {
    input: Vec<u8>,
    position: usize,
    read_position: usize,
    ch: u8,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer {
            input: input.as_bytes().to_owned(),
            position: 0,
            read_position: 0,
            ch: 0,
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = 0;
        } else {
            self.ch = self.input[self.read_position];
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn read_identifier(&mut self) -> &str {
        let start_position = self.position;
        while is_letter(self.ch) {
            self.read_char();
        }

        std::str::from_utf8(&self.input[start_position..self.position]).unwrap()
    }

    fn read_number(&mut self) -> &str {
        let start_position = self.position;
        while self.ch.is_ascii_digit() {
            self.read_char();
        }

        std::str::from_utf8(&self.input[start_position..self.position]).unwrap()
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_ascii_whitespace() {
            self.read_char();
        }
    }

    fn peek_char(&mut self) -> u8 {
        if self.read_position >= self.input.len() {
            0
        } else {
            self.input[self.read_position]
        }
    }

    fn read_string(&mut self) -> &str {
        let start_position = self.position + 1;
        loop {
            self.read_char();
            if self.ch == b'"' || self.ch == 0 {
                break;
            }
        }

        std::str::from_utf8(&self.input[start_position..self.position]).unwrap()
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        let token = match self.ch {
            b'=' => match self.peek_char() {
                b'=' => {
                    self.read_char();
                    Token::Eq
                }
                _ => Token::Assign,
            },
            b';' => Token::Semicolon,
            b'(' => Token::Lparen,
            b')' => Token::Rparen,
            b',' => Token::Comma,
            b'+' => Token::Plus,
            b'-' => Token::Minus,
            b'{' => Token::Lsquigly,
            b'}' => Token::Rsquigly,
            b'!' => match self.peek_char() {
                b'=' => {
                    self.read_char();
                    Token::NotEq
                }
                _ => Token::Bang,
            },
            b'"' => Token::String(self.read_string().to_owned()),
            b'/' => Token::Slash,
            b'*' => Token::Asterisk,
            b'<' => Token::Lt,
            b'>' => Token::Gt,
            b'\0' => Token::Eof,
            _ => {
                if is_letter(self.ch) {
                    let token = Token::lookup_ident(self.read_identifier());
                    // Exit early, because read_char() is called in the read_identifier() function.
                    return token;
                } else if self.ch.is_ascii_digit() {
                    let token = Token::Int(self.read_number().to_owned());
                    // Exit early, because read_char() is called in the read_number() function.
                    return token;
                } else {
                    Token::Illegal(self.ch)
                }
            }
        };

        self.read_char();
        token
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
"foobar"
"foo bar"
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
            Token::String("foobar".to_string()),
            Token::String("foo bar".to_string()),
            Token::Eof,
        ];

        let mut lexer = Lexer::new(input);

        for (index, expected) in expected_values.iter().enumerate() {
            let token = lexer.next_token();
            assert_eq!(
                token, *expected,
                "tests[{}] - token wrong. expected={:?}, got={:?}",
                index, expected, token
            );
        }
    }
}
