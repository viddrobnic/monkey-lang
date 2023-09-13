use std::{cell::RefCell, collections::HashMap, rc::Rc};

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

#[derive(Debug, PartialEq)]
struct EnvironmentInner {
    store: HashMap<String, Object>,
    outer: Option<Rc<RefCell<EnvironmentInner>>>,
}

impl EnvironmentInner {
    fn get(&self, name: &str) -> Option<Object> {
        if let Some(obj) = self.store.get(name) {
            return Some(obj.clone());
        }

        if let Some(outer) = &self.outer {
            return outer.borrow().get(name);
        }

        None
    }

    fn set(&mut self, name: String, value: Object) {
        self.store.insert(name, value);
    }
}

impl Clone for EnvironmentInner {
    fn clone(&self) -> Self {
        let outer = match &self.outer {
            Some(outer) => {
                let outer_clone = (*outer.borrow()).clone();
                Some(Rc::new(RefCell::new(outer_clone)))
            }
            None => None,
        };

        EnvironmentInner {
            store: self.store.clone(),
            outer,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Environment(Rc<RefCell<EnvironmentInner>>);

impl Environment {
    pub fn new() -> Self {
        Environment(Rc::new(RefCell::new(EnvironmentInner {
            store: HashMap::new(),
            outer: None,
        })))
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        let env = self.0.borrow();
        env.get(name)
    }

    pub fn set(&mut self, name: String, value: Object) {
        let mut env = self.0.borrow_mut();
        env.set(name, value);
    }

    pub fn extend(&self) -> Environment {
        Environment(Rc::new(RefCell::new(EnvironmentInner {
            store: HashMap::new(),
            outer: Some(self.0.clone()),
        })))
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for Environment {
    /// Clones the environment and all of its values.
    /// Acts as "clone by value" in contrast to the extend method
    /// which acts as "clone by reference".
    fn clone(&self) -> Self {
        let env = (*self.0.borrow()).clone();
        Environment(Rc::new(RefCell::new(env)))
    }
}

pub trait Evaluate {
    fn evaluate(&self, environment: &mut Environment) -> Result<Object>;
}
