use crate::{parse, token::Token};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PrefixOperatorKind {
    Not,
    Negative,
}

impl PrefixOperatorKind {
    pub fn debug_str(&self) -> String {
        match self {
            PrefixOperatorKind::Not => "!",
            PrefixOperatorKind::Negative => "-",
        }
        .to_owned()
    }
}

impl TryFrom<&Option<Token>> for PrefixOperatorKind {
    type Error = parse::Error;

    fn try_from(value: &Option<Token>) -> Result<Self, Self::Error> {
        match value {
            Some(Token::Bang) => Ok(Self::Not),
            Some(Token::Minus) => Ok(Self::Negative),
            token => Err(parse::Error::unexpected_token(token)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InfixOperatorKind {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
}

impl InfixOperatorKind {
    pub fn debug_str(&self) -> String {
        match self {
            InfixOperatorKind::Add => "+",
            InfixOperatorKind::Subtract => "-",
            InfixOperatorKind::Multiply => "*",
            InfixOperatorKind::Divide => "/",
            InfixOperatorKind::Equal => "==",
            InfixOperatorKind::NotEqual => "!=",
            InfixOperatorKind::GreaterThan => ">",
            InfixOperatorKind::LessThan => "<",
        }
        .to_owned()
    }
}

impl TryFrom<&Option<Token>> for InfixOperatorKind {
    type Error = parse::Error;

    fn try_from(value: &Option<Token>) -> Result<Self, Self::Error> {
        match value {
            Some(Token::Plus) => Ok(Self::Add),
            Some(Token::Minus) => Ok(Self::Subtract),
            Some(Token::Asterisk) => Ok(Self::Multiply),
            Some(Token::Slash) => Ok(Self::Divide),
            Some(Token::Eq) => Ok(Self::Equal),
            Some(Token::NotEq) => Ok(Self::NotEqual),
            Some(Token::Gt) => Ok(Self::GreaterThan),
            Some(Token::Lt) => Ok(Self::LessThan),
            token => Err(parse::Error::unexpected_token(token)),
        }
    }
}
