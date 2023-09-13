use std::collections::HashMap;

use thiserror::Error;

use crate::object::Object;

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
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    store: HashMap<String, Object>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Object> {
        self.store.get(name)
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.store.insert(name, value);
    }

    pub fn extend(&self) -> Environment {
        Environment {
            store: self.store.clone(),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Evaluate {
    fn evaluate(&self, environment: &mut Environment) -> Result<Object>;
}
