//! This package is responsible for executing the compiled bytecode.

#[cfg(test)]
mod test;

pub mod error;

mod frame;

use std::collections::HashMap;
use std::rc::Rc;

use crate::code::{Bytecode, Instruction};
use crate::object::{DataType, HashKey, Object};
pub use error::*;

use self::frame::Frame;

const STACK_SIZE: usize = 2048;
const GLOBALS_SIZE: usize = u16::MAX as usize;
const FRAME_STACK_SIZE: usize = 1024;

/// Virtual machine that can run the bytecode
#[derive(Debug)]
pub struct VirtualMachine {
    stack: Vec<Object>,
    // StackPointer which points to the next value.
    // Top of the stack is stack[sp-1]
    sp: usize,

    globals: Vec<Object>,

    frames: Vec<Option<Frame>>,
    frame_index: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            sp: 0,
            globals: vec![Object::Null; GLOBALS_SIZE],
            frames: vec![None; FRAME_STACK_SIZE],
            frame_index: 0,
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

    fn current_frame_mut(&mut self) -> &mut Frame {
        let frame = &mut self.frames[self.frame_index - 1];
        frame.as_mut().expect("Invalid frame index")
    }

    fn current_frame(&self) -> &Frame {
        let frame = &self.frames[self.frame_index - 1];
        frame.as_ref().expect("Invalid frame index")
    }

    fn push_frame(&mut self, frame: Frame) {
        self.frames[self.frame_index] = Some(frame);
        self.frame_index += 1;
    }

    fn pop_frame(&mut self) -> Frame {
        self.frame_index -= 1;

        let mut frame = None;
        std::mem::swap(&mut frame, &mut self.frames[self.frame_index]);
        frame.expect("Invalid frame index")
    }

    /// Runs the bytecode.
    ///
    /// The stack of the VM is cleaned, but the globals are left
    /// unchanged. If you don't want to keep the globals between runs,
    /// initialize a new VirtualMachine.
    pub fn run(&mut self, bytecode: &Bytecode) -> Result<()> {
        // Reinitialize the stack
        self.stack = vec![Object::Null; STACK_SIZE];
        self.sp = 0;

        // Reinitialize the frame stack
        self.frames = vec![None; FRAME_STACK_SIZE];
        self.frames[0] = Some(Frame::new(bytecode.instructions.clone(), 0));
        self.frame_index = 1;

        while self.current_frame().ip < self.current_frame().instructions.len() {
            let inst = &self.current_frame().instructions[self.current_frame().ip];
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
                    let pos = *pos;
                    let condition = self.pop();
                    if !condition.is_truthy() {
                        self.current_frame_mut().ip = pos as usize - 1;
                    }
                }
                Instruction::Jump(pos) => self.current_frame_mut().ip = *pos as usize - 1,
                Instruction::GetGlobal(idx) => self.push(self.globals[*idx as usize].clone())?,
                Instruction::SetGlobal(idx) => {
                    let idx = *idx;
                    self.globals[idx as usize] = self.pop();
                }
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
                Instruction::Index => self.execute_index_expression()?,
                Instruction::Call(num_args) => {
                    self.execute_call(*num_args as usize)?;

                    // Continue so that we don't increment the instruction
                    // pointer of the new frame.
                    continue;
                }
                Instruction::ReturnValue => {
                    let return_value = self.pop();

                    let frame = self.pop_frame();
                    self.sp = frame.base_pointer - 1; // Substract 1 to remove the function object from the stack

                    self.push(return_value)?;
                }
                Instruction::SetLocal(idx) => {
                    let frame = self.current_frame();
                    let idx = frame.base_pointer + (*idx as usize);
                    self.stack[idx] = self.pop();
                }
                Instruction::GetLocal(idx) => {
                    let frame = self.current_frame();
                    let idx = frame.base_pointer + (*idx as usize);
                    self.push(self.stack[idx].clone())?;
                }
                Instruction::GetBuiltin(bltin) => self.push(Object::Builtin(*bltin))?,
            }

            self.current_frame_mut().ip += 1
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
            left.into(),
            right.into(),
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
                DataType::String,
                DataType::String,
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
                left.into(),
                right.into(),
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
            return Err(Error::UnsupportedNegationType(operand.into()));
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

    fn execute_index_expression(&mut self) -> Result<()> {
        let index = self.pop();
        let left = self.pop();

        match left {
            Object::Array(arr) => self.execute_array_index(&arr, index),
            Object::HashMap(hash) => self.execute_hash_index(&hash, index),
            _ => Err(Error::IndexOperatorNotSupported(left.into(), index.into())),
        }
    }

    fn execute_array_index(&mut self, arr: &[Object], index: Object) -> Result<()> {
        let Object::Integer(idx) = index else {
            return Err(Error::IndexOperatorNotSupported(
                DataType::Array,
                index.into(),
            ));
        };

        if idx < 0 {
            self.push(Object::Null)?;
            return Ok(());
        }
        if (idx as usize) >= arr.len() {
            self.push(Object::Null)?;
            return Ok(());
        }

        self.push(arr[idx as usize].clone())?;
        Ok(())
    }

    fn execute_hash_index(&mut self, hash: &HashMap<HashKey, Object>, index: Object) -> Result<()> {
        let key: HashKey = index.try_into().map_err(Error::UnhashableKey)?;

        let obj = hash.get(&key).unwrap_or(&Object::Null);
        self.push(obj.clone())?;

        Ok(())
    }

    fn execute_call(&mut self, num_args: usize) -> Result<()> {
        match &self.stack[self.sp - num_args - 1] {
            Object::CompiledFunction {
                instructions,
                num_locals,
                num_arguments,
            } => {
                if num_args != *num_arguments {
                    println!("{:?}", &self.stack[0..30]);
                    return Err(Error::WrongNumberOfArguments {
                        want: *num_arguments,
                        got: num_args,
                    });
                }

                let frame = Frame::new(instructions.clone(), self.sp - num_args);
                self.sp = frame.base_pointer + *num_locals;
                self.push_frame(frame);

                Ok(())
            }
            Object::Builtin(fun) => {
                let args = &self.stack[(self.sp - num_args)..self.sp];
                let result = fun.execute(args)?;

                self.sp = self.sp - num_args - 1;

                self.push(result)?;
                self.current_frame_mut().ip += 1;

                Ok(())
            }
            obj => Err(Error::NotAFunction(obj.into())),
        }
    }
}
