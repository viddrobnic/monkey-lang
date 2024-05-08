use std::rc::Rc;

use crate::code::Instruction;

#[derive(Debug, Clone)]
pub struct Frame {
    pub instructions: Rc<Vec<Instruction>>,
    pub ip: usize,
    pub base_pointer: usize,
}

impl Frame {
    pub fn new(instructions: Rc<Vec<Instruction>>, base_pointer: usize) -> Self {
        Self {
            instructions,
            ip: 0,
            base_pointer,
        }
    }
}
