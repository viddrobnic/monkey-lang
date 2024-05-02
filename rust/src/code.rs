//! Bytecode implementation

use crate::object;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    Constant(u16),
}

#[derive(Debug, PartialEq)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<object::Object>,
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}
