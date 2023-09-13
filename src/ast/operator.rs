use crate::{parse::Error, token::Token};

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

impl TryFrom<&Token> for PrefixOperatorKind {
    type Error = Error;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value {
            Token::Bang => Ok(Self::Not),
            Token::Minus => Ok(Self::Negative),
            token => Err(Error::UnexpectedToken(token.clone())),
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

impl TryFrom<&Token> for InfixOperatorKind {
    type Error = Error;

    fn try_from(value: &Token) -> Result<Self, Self::Error> {
        match value {
            Token::Plus => Ok(Self::Add),
            Token::Minus => Ok(Self::Subtract),
            Token::Asterisk => Ok(Self::Multiply),
            Token::Slash => Ok(Self::Divide),
            Token::Eq => Ok(Self::Equal),
            Token::NotEq => Ok(Self::NotEqual),
            Token::Gt => Ok(Self::GreaterThan),
            Token::Lt => Ok(Self::LessThan),
            token => Err(Error::UnexpectedToken(token.clone())),
        }
    }
}
