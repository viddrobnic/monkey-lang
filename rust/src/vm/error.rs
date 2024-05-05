use thiserror::Error;

use crate::{code::Instruction, object::DataType};

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("stack overflow")]
    StackOverflow,
    #[error("unknown binary operator: {0:?} ({1}, {2})")]
    UnknownBinaryOperator(Instruction, DataType, DataType),
    #[error("unsupported type for negation: {0}")]
    UnsupportedNegationType(DataType),
    #[error("key not hashable: {0}")]
    UnhashableKey(DataType),
    #[error("index operator not supported: {0}[{1}]")]
    IndexOperatorNotSupported(DataType, DataType),
}

pub type Result<T> = std::result::Result<T, Error>;
