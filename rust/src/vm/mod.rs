//! This package is responsible for executing the compiled bytecode.

#[cfg(test)]
mod test;

pub mod error;

use std::collections::HashMap;
use std::rc::Rc;

use crate::code::{Bytecode, Instruction};
use crate::object::{HashKey, Object};
pub use error::*;

const STACK_SIZE: usize = 2048;
const GLOBALS_SIZE: usize = u16::MAX as usize;

/// Virtual machine that can run the bytecode
pub struct VirtualMachine {
    stack: Vec<Object>,
    // StackPointer which points to the next value.
    // Top of the stack is stack[sp-1]
    sp: usize,

    globals: Vec<Object>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            sp: 0,
            globals: vec![Object::Null; GLOBALS_SIZE],
        }
    }
}

impl Default for VirtualMachine {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualMachine {
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

    /// Runs the bytecode.
    ///
    /// The stack of the VM is cleaned, but the globals are left
    /// unchanged. If you don't want to keep the globals between runs,
    /// initialize a new VirtualMachine.
    pub fn run(&mut self, bytecode: &Bytecode) -> Result<()> {
        self.stack = vec![Object::Null; STACK_SIZE];
        self.sp = 0;

        let mut ip = 0;
        while ip < bytecode.instructions.len() {
            let inst = &bytecode.instructions[ip];
            match inst {
                Instruction::Constant(idx) => {
                    self.push(bytecode.constants[*idx as usize].clone())?
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
                Instruction::Array(len) => {
                    let length = *len as usize;
                    let start = self.sp - length;

                    let arr = self.stack[start..self.sp].to_vec();

                    self.sp -= length;
                    self.push(Object::Array(Rc::new(arr)))?;
                }
                Instruction::Hash(len) => {
                    let length = *len as usize;

                    let hash_map = self.build_hash_map(length)?;

                    self.sp -= length;
                    self.push(hash_map)?;
                }
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

        if let (Object::String(left), Object::String(right)) = (&left, &right) {
            return self.execute_binary_string_operation(instruction, left, right);
        }

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

    fn execute_binary_string_operation(
        &mut self,
        operation: Instruction,
        left: &str,
        right: &str,
    ) -> Result<()> {
        if operation != Instruction::Add {
            return Err(Error::UnknownBinaryOperator(
                Instruction::Add,
                "STRING".to_string(),
                "STRING".to_string(),
            ));
        }

        let res = String::from(left) + right;
        self.push(Object::String(Rc::new(res)))
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

    fn build_hash_map(&self, length: usize) -> Result<Object> {
        let start = self.sp - length;

        let hash_map: Result<HashMap<_, _>> = self.stack[start..self.sp]
            .chunks(2)
            .map(|chunk| -> Result<(HashKey, Object)> {
                let key = &chunk[0];
                let value = &chunk[1];

                let key: HashKey = key.clone().try_into().map_err(Error::UnhashableKey)?;

                Ok((key, value.clone()))
            })
            .collect();

        hash_map.map(|hm| Object::HashMap(Rc::new(hm)))
    }
}
