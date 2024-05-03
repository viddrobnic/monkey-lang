#[cfg(test)]
mod test;

use crate::ast;
use crate::code::{Bytecode, Instruction};
use crate::object::Object;

pub fn compile(program: &ast::Program) -> Bytecode {
    let mut compiler = Compiler::new();
    compiler.compile(program);
    compiler.bytecode
}

struct Compiler {
    bytecode: Bytecode,
}

impl Compiler {
    fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
        }
    }

    fn add_constant(&mut self, obj: Object) -> usize {
        self.bytecode.constants.push(obj);
        self.bytecode.constants.len() - 1
    }

    fn emit(&mut self, instruction: Instruction) -> usize {
        self.bytecode.instructions.push(instruction);
        self.bytecode.instructions.len() - 1
    }

    fn compile(&mut self, program: &ast::Program) {
        for stmt in &program.statements {
            self.compile_statement(stmt);
        }
    }

    fn compile_statement(&mut self, statement: &ast::Statement) {
        match statement {
            ast::Statement::Let { .. } => todo!(),
            ast::Statement::Return(_) => todo!(),
            ast::Statement::Expression(expr) => {
                self.compile_expression(expr);
                self.emit(Instruction::Pop);
            }
        }
    }

    fn compile_expression(&mut self, expression: &ast::Expression) {
        match expression {
            ast::Expression::Identifier(_) => todo!(),
            ast::Expression::IntegerLiteral(val) => {
                let const_idx = self.add_constant(Object::Integer(*val));
                self.emit(Instruction::Constant(const_idx as u16));
            }
            ast::Expression::BooleanLiteral(val) => {
                if *val {
                    self.emit(Instruction::True);
                } else {
                    self.emit(Instruction::False);
                }
            }
            ast::Expression::StringLiteral(_) => todo!(),
            ast::Expression::ArrayLiteral(_) => todo!(),
            ast::Expression::HashLiteral(_) => todo!(),
            ast::Expression::PrefixOperator { operator, right } => {
                self.compile_expression(right);
                match operator {
                    ast::PrefixOperatorKind::Not => self.emit(Instruction::Bang),
                    ast::PrefixOperatorKind::Negative => self.emit(Instruction::Minus),
                };
            }
            ast::Expression::InfixOperator {
                left,
                right,
                operator,
            } => {
                if *operator == ast::InfixOperatorKind::LessThan {
                    self.compile_expression(right);
                    self.compile_expression(left);
                    self.emit(Instruction::GreaterThan);
                    return;
                }

                self.compile_expression(left);
                self.compile_expression(right);

                match operator {
                    ast::InfixOperatorKind::Add => self.emit(Instruction::Add),
                    ast::InfixOperatorKind::Subtract => self.emit(Instruction::Sub),
                    ast::InfixOperatorKind::Multiply => self.emit(Instruction::Mul),
                    ast::InfixOperatorKind::Divide => self.emit(Instruction::Div),
                    ast::InfixOperatorKind::Equal => self.emit(Instruction::Equal),
                    ast::InfixOperatorKind::NotEqual => self.emit(Instruction::NotEqual),
                    ast::InfixOperatorKind::GreaterThan => self.emit(Instruction::GreaterThan),
                    ast::InfixOperatorKind::LessThan => unreachable!(),
                };
            }
            ast::Expression::If { .. } => todo!(),
            ast::Expression::FunctionLiteral { .. } => todo!(),
            ast::Expression::FunctionCall { .. } => todo!(),
            ast::Expression::Index { .. } => todo!(),
        }
    }
}
