use thiserror::Error;

use crate::code::Instruction;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("stack overflow")]
    StackOverflow,
    #[error("unknown binary operator: {0:?} ({1} {2})")]
    UnknownBinaryOperator(Instruction, String, String),
    #[error("unsupported type for negation: {0}")]
    UnsupportedNegationType(String),
}

pub type Result<T> = std::result::Result<T, Error>;
