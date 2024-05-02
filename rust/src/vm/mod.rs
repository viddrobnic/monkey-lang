#[cfg(test)]
mod test;

use crate::code::{Bytecode, Instruction};
use crate::object::Object;

const STACK_SIZE: usize = 2048;

pub struct VirtualMachine<'a> {
    bytecode: &'a Bytecode,

    stack: Vec<Object>,
    // StackPointer which points to the next value.
    // Top of the stack is stack[sp-1]
    sp: usize,
}

impl<'a> VirtualMachine<'a> {
    pub fn new(bytecode: &'a Bytecode) -> Self {
        Self {
            bytecode,
            stack: vec![Object::Null; STACK_SIZE],
            sp: 0,
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

    fn push(&mut self, obj: Object) {
        if self.sp >= self.stack.len() {
            // TODO: Return error
            panic!("Stack overflow");
        }

        self.stack[self.sp] = obj;
        self.sp += 1;
    }

    fn pop(&mut self) -> Object {
        self.sp -= 1;
        self.stack[self.sp].clone()
    }

    pub fn run(&mut self) {
        for inst in &self.bytecode.instructions {
            match inst {
                Instruction::Constant(idx) => {
                    self.push(self.bytecode.constants[*idx as usize].clone());
                }
                Instruction::Add | Instruction::Mul | Instruction::Sub | Instruction::Div => {
                    self.execute_binary_operation(*inst);
                }
                Instruction::Pop => {
                    self.pop();
                }
            }
        }
    }

    fn execute_binary_operation(&mut self, instruction: Instruction) {
        let right = self.pop();
        let left = self.pop();

        if let (Object::Integer(left), Object::Integer(right)) = (&left, &right) {
            self.execute_binary_integer_operation(instruction, *left, *right);
            return;
        };

        // TODO: Use errors
        panic!(
            "unsupported types for binary operations: {}, {}",
            left.data_type(),
            right.data_type()
        );
    }

    fn execute_binary_integer_operation(&mut self, operation: Instruction, left: i64, right: i64) {
        let res = match operation {
            Instruction::Add => left + right,
            Instruction::Sub => left - right,
            Instruction::Mul => left * right,
            Instruction::Div => left / right,
            _ => unreachable!(),
        };

        self.push(Object::Integer(res))
    }
}
