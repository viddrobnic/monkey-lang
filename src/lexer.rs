use crate::token::Token;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: None,
        };
        lexer.read_char();
        lexer
    }

    fn read_char(&mut self) {
        if self.read_position == self.input.len() {
            self.ch = Some('\0');
        } else if self.read_position > self.input.len() {
            self.ch = None;
        } else {
            self.ch = Some(self.input[self.read_position]);
        }

        self.position = self.read_position;
        self.read_position += 1;
    }

    fn read_identifier(&mut self) -> String {
        let start_position = self.position;
        while self.ch.is_some_and(is_letter) {
            self.read_char();
        }

        self.input[start_position..self.position].iter().collect()
    }

    fn read_number(&mut self) -> String {
        let start_position = self.position;
        while self.ch.is_some_and(|ch| ch.is_ascii_digit()) {
            self.read_char();
        }

        self.input[start_position..self.position].iter().collect()
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_some_and(|ch| ch.is_whitespace()) {
            self.read_char();
        }
    }

    fn peek_char(&mut self) -> Option<char> {
        if self.read_position >= self.input.len() {
            None
        } else {
            Some(self.input[self.read_position])
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        let ch = match self.ch {
            Some(ch) => ch,
            None => return None,
        };

        let token = match ch {
            '=' => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    Token::Eq
                } else {
                    Token::Assign
                }
            }
            ';' => Token::Semicolon,
            '(' => Token::Lparen,
            ')' => Token::Rparen,
            ',' => Token::Comma,
            '+' => Token::Plus,
            '-' => Token::Minus,
            '{' => Token::Lsquigly,
            '}' => Token::Rsquigly,
            '!' => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    Token::NotEq
                } else {
                    Token::Bang
                }
            }
            '/' => Token::Slash,
            '*' => Token::Asterisk,
            '<' => Token::Lt,
            '>' => Token::Gt,
            '\0' => Token::Eof,
            _ => {
                if is_letter(ch) {
                    let token = Token::lookup_ident(&self.read_identifier());
                    // Exit early, because read_char() is called in the read_identifier() function.
                    return Some(token);
                } else if ch.is_ascii_digit() {
                    let token = Token::Int(self.read_number());
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

fn is_letter(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_'
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
