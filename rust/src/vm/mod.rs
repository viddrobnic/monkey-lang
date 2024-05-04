#[cfg(test)]
mod test;

pub mod error;

use crate::code::{Bytecode, Instruction};
use crate::object::Object;
pub use error::*;

const STACK_SIZE: usize = 2048;
const GLOBALS_SIZE: usize = u16::MAX as usize;

pub struct VirtualMachine<'a> {
    bytecode: &'a Bytecode,

    stack: Vec<Object>,
    // StackPointer which points to the next value.
    // Top of the stack is stack[sp-1]
    sp: usize,

    globals: Vec<Object>,
}

impl<'a> VirtualMachine<'a> {
    pub fn new(bytecode: &'a Bytecode) -> Self {
        Self {
            bytecode,
            stack: vec![Object::Null; STACK_SIZE],
            sp: 0,
            globals: vec![Object::Null; GLOBALS_SIZE],
        }
    }
}

impl VirtualMachine<'_> {
    pub fn stack_top(&self) -> Option<&Object> {
        if self.sp == 0 {
            None
        } else {
            Some(&self.stack[self.sp - 1])
        }
    }

    pub fn last_popped(&self) -> &Object {
        &self.stack[self.sp]
    }

    fn push(&mut self, obj: Object) -> Result<()> {
        if self.sp >= self.stack.len() {
            return Err(Error::StackOverflow);
        }

        self.stack[self.sp] = obj;
        self.sp += 1;
        Ok(())
    }

    fn pop(&mut self) -> Object {
        self.sp -= 1;
        self.stack[self.sp].clone()
    }

    pub fn run(&mut self) -> Result<()> {
        let mut ip = 0;
        while ip < self.bytecode.instructions.len() {
            let inst = &self.bytecode.instructions[ip];
            match inst {
                Instruction::Constant(idx) => {
                    self.push(self.bytecode.constants[*idx as usize].clone())?
                }
                Instruction::Add | Instruction::Mul | Instruction::Sub | Instruction::Div => {
                    self.execute_binary_operation(*inst)?;
                }
                Instruction::Equal | Instruction::NotEqual | Instruction::GreaterThan => {
                    self.execute_comparison(*inst)?;
                }
                Instruction::True => self.push(Object::Boolean(true))?,
                Instruction::False => self.push(Object::Boolean(false))?,
                Instruction::Null => self.push(Object::Null)?,
                Instruction::Pop => {
                    self.pop();
                }
                Instruction::Bang => self.execute_bang_operator()?,
                Instruction::Minus => self.execute_minus_operator()?,
                Instruction::JumpNotTruthy(pos) => {
                    let condition = self.pop();
                    if !condition.is_truthy() {
                        ip = *pos as usize - 1;
                    }
                }
                Instruction::Jump(pos) => ip = *pos as usize - 1,
                Instruction::GetGlobal(idx) => self.push(self.globals[*idx as usize].clone())?,
                Instruction::SetGlobal(idx) => self.globals[*idx as usize] = self.pop(),
            }

            ip += 1
        }

        Ok(())
    }

    fn execute_binary_operation(&mut self, instruction: Instruction) -> Result<()> {
        let right = self.pop();
        let left = self.pop();

        if let (Object::Integer(left), Object::Integer(right)) = (&left, &right) {
            return self.execute_binary_integer_operation(instruction, *left, *right);
        };

        Err(Error::UnknownBinaryOperator(
            instruction,
            left.data_type().to_string(),
            right.data_type().to_string(),
        ))
    }

    fn execute_binary_integer_operation(
        &mut self,
        operation: Instruction,
        left: i64,
        right: i64,
    ) -> Result<()> {
        let res = match operation {
            Instruction::Add => left + right,
            Instruction::Sub => left - right,
            Instruction::Mul => left * right,
            Instruction::Div => left / right,
            _ => unreachable!(),
        };

        self.push(Object::Integer(res))
    }

    fn execute_comparison(&mut self, instruction: Instruction) -> Result<()> {
        let right = self.pop();
        let left = self.pop();

        if let (Object::Integer(left), Object::Integer(right)) = (&left, &right) {
            return self.execute_integer_comparison(instruction, *left, *right);
        }

        match instruction {
            Instruction::Equal => self.push(Object::Boolean(left == right)),
            Instruction::NotEqual => self.push(Object::Boolean(left != right)),
            _ => Err(Error::UnknownBinaryOperator(
                instruction,
                left.data_type().to_string(),
                right.data_type().to_string(),
            )),
        }
    }

    fn execute_integer_comparison(
        &mut self,
        operation: Instruction,
        left: i64,
        right: i64,
    ) -> Result<()> {
        let res = match operation {
            Instruction::Equal => left == right,
            Instruction::NotEqual => left != right,
            Instruction::GreaterThan => left > right,
            _ => unreachable!(),
        };

        self.push(Object::Boolean(res))
    }

    fn execute_bang_operator(&mut self) -> Result<()> {
        let operand = self.pop();
        self.push(Object::Boolean(!operand.is_truthy()))
    }

    fn execute_minus_operator(&mut self) -> Result<()> {
        let operand = self.pop();

        let Object::Integer(value) = operand else {
            return Err(Error::UnsupportedNegationType(
                operand.data_type().to_string(),
            ));
        };

        self.push(Object::Integer(-value))
    }
}
