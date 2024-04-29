#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Illegal(u8),
    // Identifiers + literals
    Ident(String), // add, foobar, x, y, ...
    Int(String),   // 1343456
    String(String),
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
    Colon,
    Lparen,
    Rparen,
    Lsquigly,
    Rsquigly,
    LBracket,
    RBracket,
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

    pub fn is_infix(&self) -> bool {
        matches!(
            self,
            Self::Plus
                | Self::Minus
                | Self::Slash
                | Self::Asterisk
                | Self::Eq
                | Self::NotEq
                | Self::Lt
                | Self::Gt
                | Self::Lparen
                | Self::LBracket
        )
    }
}
