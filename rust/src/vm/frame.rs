use std::rc::Rc;

use crate::code::Instruction;

#[derive(Debug, Clone)]
pub struct Frame {
    pub instructions: Rc<Vec<Instruction>>,
    pub ip: usize,
}

impl Frame {
    pub fn new(instructions: Rc<Vec<Instruction>>) -> Self {
        Self {
            instructions,
            ip: 0,
        }
    }
}
