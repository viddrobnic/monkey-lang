use thiserror::Error;

use super::{builtin, DataType};

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
    #[error("index operator not supported: {0}[{1}]")]
    IndexOperatorNotSupported(DataType, DataType),
    #[error("not hashable: {0}")]
    NotHashable(DataType),
    #[error("builtin function error: {source}")]
    BuiltinFunction {
        #[from]
        source: builtin::ExecutionError,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
