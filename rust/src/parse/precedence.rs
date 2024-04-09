use crate::token::Token;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
    Index,
}

impl From<&Token> for Precedence {
    fn from(value: &Token) -> Self {
        match value {
            Token::Eq | Token::NotEq => Self::Equals,
            Token::Lt | Token::Gt => Self::LessGreater,
            Token::Plus | Token::Minus => Self::Sum,
            Token::Slash | Token::Asterisk => Self::Product,
            Token::Lparen => Self::Call,
            Token::LBracket => Self::Index,
            _ => Self::Lowest,
        }
    }
}

impl From<&Option<Token>> for Precedence {
    fn from(value: &Option<Token>) -> Self {
        match value {
            Some(token) => token.into(),
            None => Precedence::Lowest,
        }
    }
}
