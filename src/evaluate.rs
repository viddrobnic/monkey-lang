use thiserror::Error;

use crate::object::Object;

#[derive(Debug, Error, PartialEq)]
pub enum Error {
    #[error("type mismatch: {0}")]
    TypeMismatch(String),
    #[error("unknown operator: {0}")]
    UnknownOperator(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Evaluate {
    fn evaluate(&self) -> Result<Object>;
}
