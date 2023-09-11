use std::num::ParseIntError;

use thiserror::Error;

use crate::token::Token;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unexpected token: {0:?}")]
    UnexpectedToken(Token),
    #[error("Expected a statement, got: {0:?}")]
    NotAStatement(Token),
    #[error("Expected an expression, got: {0:?}")]
    NotAnExpression(Token),
    #[error(transparent)]
    NotANumber(#[from] ParseIntError),
    #[error("Expected a left expression, got None")]
    ExpectedLeftExpression,
}
