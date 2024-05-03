use thiserror::Error;

use crate::code::Instruction;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("stack overflow")]
    StackOverflow,
    #[error("unknown operator: {0:?} ({1} {2})")]
    UnknownOperator(Instruction, String, String),
}

pub type Result<T> = std::result::Result<T, Error>;
