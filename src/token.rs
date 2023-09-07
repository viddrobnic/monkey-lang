#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Illegal(u8),
    Eof,
    // Identifiers + literals
    Ident(String), // add, foobar, x, y, ...
    Int(String),   // 1343456
    // Operators
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    Lt,
    Gt,
    Eq,
    NotEq,
    // Delimiters
    Comma,
    Semicolon,
    Lparen,
    Rparen,
    Lsquigly,
    Rsquigly,
    // Keywords
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}

impl Token {
    pub fn lookup_ident(ident: &str) -> Token {
        match ident {
            "fn" => Token::Function,
            "let" => Token::Let,
            "true" => Token::True,
            "false" => Token::False,
            "if" => Token::If,
            "else" => Token::Else,
            "return" => Token::Return,
            _ => Token::Ident(ident.to_string()),
        }
    }
}
