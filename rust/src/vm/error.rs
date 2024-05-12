use thiserror::Error;

use crate::{
    code::Instruction,
    object::{builtin, DataType},
};

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
    #[error("calling non-closure and non-builtin {0}")]
    NotCallable(DataType),
    #[error("not a function {0}")]
    NotAFunction(DataType),
    #[error("wrong number of arguments, want: {want}, got: {got}")]
    WrongNumberOfArguments { want: usize, got: usize },
    #[error("builtin function error: {source}")]
    BuiltinFunction {
        #[from]
        source: builtin::ExecutionError,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
