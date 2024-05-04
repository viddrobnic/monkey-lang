use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("undefined symbol: {0}")]
    UndefinedSymbol(String),
}

pub type Result<T> = std::result::Result<T, Error>;
