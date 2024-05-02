#[cfg(test)]
mod test;

use crate::ast;
use crate::code::{Bytecode, Instruction};
use crate::object::Object;

pub struct Compiler {
    bytecode: Bytecode,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
        }
    }

    pub fn bytecode(&self) -> &Bytecode {
        &self.bytecode
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    fn add_constant(&mut self, obj: Object) -> usize {
        self.bytecode.constants.push(obj);
        self.bytecode.constants.len() - 1
    }

    fn emit(&mut self, instruction: Instruction) -> usize {
        self.bytecode.instructions.push(instruction);
        self.bytecode.instructions.len() - 1
    }

    pub fn compile(&mut self, program: &ast::Program) {
        for stmt in &program.statements {
            self.compile_statement(stmt);
        }
    }

    fn compile_statement(&mut self, statement: &ast::Statement) {
        match statement {
            ast::Statement::Let { .. } => todo!(),
            ast::Statement::Return(_) => todo!(),
            ast::Statement::Expression(expr) => self.compile_expression(expr),
        }
    }

    fn compile_expression(&mut self, expression: &ast::Expression) {
        match expression {
            ast::Expression::Identifier(_) => todo!(),
            ast::Expression::IntegerLiteral(val) => {
                let const_idx = self.add_constant(Object::Integer(*val));
                self.emit(Instruction::Constant(const_idx as u16));
            }
            ast::Expression::BooleanLiteral(_) => todo!(),
            ast::Expression::StringLiteral(_) => todo!(),
            ast::Expression::ArrayLiteral(_) => todo!(),
            ast::Expression::HashLiteral(_) => todo!(),
            ast::Expression::PrefixOperator { .. } => todo!(),
            ast::Expression::InfixOperator { left, right, .. } => {
                self.compile_expression(left);
                self.compile_expression(right);
            }
            ast::Expression::If { .. } => todo!(),
            ast::Expression::FunctionLiteral { .. } => todo!(),
            ast::Expression::FunctionCall { .. } => todo!(),
            ast::Expression::Index { .. } => todo!(),
        }
    }
}
