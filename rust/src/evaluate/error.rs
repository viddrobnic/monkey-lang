use thiserror::Error;

use super::DataType;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("type mismatch: {0}")]
    TypeMismatch(String),
    #[error("unknown operator: {0}")]
    UnknownOperator(String),
    #[error("identifier not found: {0}")]
    UnknownIdentifier(String),
    #[error("not a function: {0}")]
    NotAFunction(DataType),
    #[error("wrong number of arguments: expected {expected}, got {got}")]
    WrongNumberOfArguments { expected: usize, got: usize },
    #[error("index operator not supported: {0}[{1}]")]
    IndexOperatorNotSupported(DataType, DataType),
    #[error("not hashable: {0}")]
    NotHashable(DataType),
}

pub type Result<T> = std::result::Result<T, Error>;
