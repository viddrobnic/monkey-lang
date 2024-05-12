use crate::object;

#[derive(Debug, Clone)]
pub struct Frame {
    pub closure: object::Closure,
    pub ip: usize,
    pub base_pointer: usize,
}

impl Frame {
    pub fn new(closure: object::Closure, base_pointer: usize) -> Self {
        Self {
            closure,
            ip: 0,
            base_pointer,
        }
    }
}
