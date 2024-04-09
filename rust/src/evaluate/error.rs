use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("type mismatch: {0}")]
    TypeMismatch(String),
    #[error("unknown operator: {0}")]
    UnknownOperator(String),
    #[error("identifier not found: {0}")]
    UnknownIdentifier(String),
    #[error("not a function: {0}")]
    NotAFunction(String),
    #[error("wrong number of arguments: expected {expected}, got {got}")]
    WrongNumberOfArguments { expected: usize, got: usize },
}

pub type Result<T> = std::result::Result<T, Error>;
