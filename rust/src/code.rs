//! Bytecode implementation

use std::rc::Rc;

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

    Call,
    ReturnValue,
}

#[derive(Debug, PartialEq)]
pub struct Bytecode<'a> {
    pub instructions: Rc<Vec<Instruction>>,
    pub constants: &'a [object::Object],
}
