use crate::token::Token;
use std::num::ParseIntError;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token),
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Expected a statement, got: {0:?}")]
    NotAStatement(Token),
    #[error("Expected an expression, got: {0:?}")]
    NotAnExpression(Option<Token>),
    #[error(transparent)]
    NotANumber(#[from] ParseIntError),
    #[error("Expected a left expression, got None")]
    ExpectedLeftExpression,
}

impl Error {
    pub fn unexpected_token(token: &Option<Token>) -> Self {
        match token {
            Some(token) => Self::UnexpectedToken(token.clone()),
            None => Self::UnexpectedEof,
        }
    }
}
