//! Bytecode implementation

use crate::object;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Instruction {
    Constant(u16),

    Add,
    Sub,
    Mul,
    Div,

    Pop,

    Null,
    True,
    False,

    Equal,
    NotEqual,
    GreaterThan,

    Minus,
    Bang,

    JumpNotTruthy(u16),
    Jump(u16),

    GetGlobal(u16),
    SetGlobal(u16),

    Array(u16),
    Hash(u16),
    Index,
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
