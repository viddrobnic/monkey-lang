use std::rc::{Rc, Weak};
use std::{cell::RefCell, collections::HashMap};

use super::Object;

#[derive(Debug)]
struct EnvironmentInner {
    store: HashMap<String, Object>,
    outer: Option<Weak<RefCell<EnvironmentInner>>>,
}

impl EnvironmentInner {
    fn get(&self, name: &str) -> Option<Object> {
        if let Some(obj) = self.store.get(name) {
            return Some(obj.clone());
        }

        if let Some(outer) = &self.outer {
            return outer
                .upgrade()
                .expect("Trying to access a dropped environment")
                .borrow()
                .get(name);
        }

        None
    }

    fn set(&mut self, name: String, value: Object) {
        self.store.insert(name, value);
    }
}

impl PartialEq for EnvironmentInner {
    fn eq(&self, other: &Self) -> bool {
        let outer_eq = match (&self.outer, &other.outer) {
            (Some(outer1), Some(outer2)) => {
                let outer1 = outer1
                    .upgrade()
                    .expect("Trying to access a dropped environment");
                let outer2 = outer2
                    .upgrade()
                    .expect("Trying to access a dropped environment");
                outer1 == outer2
            }
            (None, None) => true,
            _ => false,
        };
        self.store == other.store && outer_eq
    }
}

#[derive(Debug, Clone)]
pub struct Environment(Weak<RefCell<EnvironmentInner>>);

/// Owner of the environment. Once this is dropped,
/// the environment is dropped as well.
/// Trying to access the environment after this is dropped
/// will result in a panic.
#[derive(Debug)]
pub struct EnvironmentOwner(Rc<RefCell<EnvironmentInner>>);

impl PartialEq for EnvironmentOwner {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for EnvironmentOwner {}

impl std::hash::Hash for EnvironmentOwner {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state)
    }
}

impl Environment {
    pub fn new() -> (Self, EnvironmentOwner) {
        let env = Rc::new(RefCell::new(EnvironmentInner {
            store: HashMap::new(),
            outer: None,
        }));

        let env_weak = Rc::downgrade(&env);

        (Environment(env_weak), EnvironmentOwner(env))
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        return self
            .0
            .upgrade()
            .expect("Trying to access a dropped environment")
            .borrow()
            .get(name);
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.0
            .upgrade()
            .expect("Trying to access a dropped environment")
            .borrow_mut()
            .set(name, value);
    }

    pub fn extend(&self) -> (Environment, EnvironmentOwner) {
        let env = Rc::new(RefCell::new(EnvironmentInner {
            store: HashMap::new(),
            outer: Some(self.0.clone()),
        }));

        let env_weak = Rc::downgrade(&env);

        (Environment(env_weak), EnvironmentOwner(env))
    }

    pub fn is_owned_by(&self, owner: &EnvironmentOwner) -> bool {
        let Some(self_rc) = self.0.upgrade() else {
            return false;
        };

        Rc::ptr_eq(&self_rc, &owner.0)
    }
}

impl PartialEq for Environment {
    fn eq(&self, other: &Self) -> bool {
        let self_rc = self
            .0
            .upgrade()
            .expect("Trying to access a dropped environment");
        let other_rc = other
            .0
            .upgrade()
            .expect("Trying to access a dropped environment");

        self_rc == other_rc
    }
}
