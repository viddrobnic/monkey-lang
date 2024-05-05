//! This package is responsible for compiling
//! the AST to the bytecode that can be executed
//! by the VM.

#[cfg(test)]
mod test;

pub mod error;

mod symbol_table;

use std::rc::Rc;

use crate::ast;
use crate::code::{Bytecode, Instruction};
use crate::object::Object;

use self::symbol_table::SymbolTable;

pub use error::*;

/// Compiles AST to the bytecode.
pub struct Compiler {
    bytecode: Bytecode,
    symbol_table: SymbolTable,
}

impl Compiler {
    /// Creates a new compiler with empty state
    /// (globals, constants, ...).
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            symbol_table: SymbolTable::new(),
        }
    }

    /// Compiles a program.
    ///
    /// The instructions part of the bytecode is overriden,
    /// but the state (globals, constants, ...) is left unchanged.
    /// If you don't want to keep the state between compilations,
    /// initialize a new compiler.
    pub fn compile(&mut self, program: &ast::Program) -> Result<()> {
        self.bytecode.instructions = vec![];

        for stmt in &program.statements {
            self.compile_statement(stmt)?;
        }

        Ok(())
    }

    /// Returns the current bytecode.
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

    fn compile_statement(&mut self, statement: &ast::Statement) -> Result<()> {
        match statement {
            ast::Statement::Let { name, value } => {
                self.compile_expression(value)?;

                let symbol = self.symbol_table.define(name.clone());
                self.emit(Instruction::SetGlobal(symbol.index));
            }
            ast::Statement::Return(_) => todo!(),
            ast::Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                self.emit(Instruction::Pop);
            }
        }

        Ok(())
    }

    fn compile_expression(&mut self, expression: &ast::Expression) -> Result<()> {
        match expression {
            ast::Expression::Identifier(ident) => {
                let symbol = self.symbol_table.resolve(ident);
                match symbol {
                    Some(symbol) => {
                        self.emit(Instruction::GetGlobal(symbol.index));
                    }
                    None => return Err(Error::UndefinedSymbol(ident.to_string())),
                }
            }
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
            ast::Expression::StringLiteral(string) => {
                let const_idx = self.add_constant(Object::String(Rc::new(string.clone())));
                self.emit(Instruction::Constant(const_idx as u16));
            }
            ast::Expression::ArrayLiteral(arr) => {
                for expr in arr {
                    self.compile_expression(expr)?;
                }

                self.emit(Instruction::Array(arr.len() as u16));
            }
            ast::Expression::HashLiteral(hash) => {
                for pair in hash {
                    self.compile_expression(&pair.key)?;
                    self.compile_expression(&pair.value)?;
                }

                let length = (hash.len() * 2) as u16;
                self.emit(Instruction::Hash(length));
            }
            ast::Expression::PrefixOperator { operator, right } => {
                self.compile_expression(right)?;
                match operator {
                    ast::PrefixOperatorKind::Not => self.emit(Instruction::Bang),
                    ast::PrefixOperatorKind::Negative => self.emit(Instruction::Minus),
                };
            }
            ast::Expression::InfixOperator { .. } => self.compile_infix_operator(expression)?,
            ast::Expression::If { .. } => self.compile_conditional(expression)?,
            ast::Expression::FunctionLiteral { .. } => todo!(),
            ast::Expression::FunctionCall { .. } => todo!(),
            ast::Expression::Index { .. } => todo!(),
        }

        Ok(())
    }

    fn compile_block_statement(&mut self, statement: &ast::BlockStatement) -> Result<()> {
        if statement.statements.len() == 0 {
            self.emit(Instruction::Null);
            return Ok(());
        }

        for stmt in statement.statements.iter() {
            self.compile_statement(stmt)?;
        }

        Ok(())
    }

    fn compile_infix_operator(&mut self, expression: &ast::Expression) -> Result<()> {
        let ast::Expression::InfixOperator {
            operator,
            left,
            right,
        } = expression
        else {
            panic!("Expected InfixOperator expression, got: {:?}", expression);
        };

        // Handle LessThan as special case, since VM supports only
        // GreaterThan instruction.
        if *operator == ast::InfixOperatorKind::LessThan {
            self.compile_expression(right)?;
            self.compile_expression(left)?;
            self.emit(Instruction::GreaterThan);
            return Ok(());
        }

        self.compile_expression(left)?;
        self.compile_expression(right)?;

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

        Ok(())
    }

    fn compile_conditional(&mut self, expression: &ast::Expression) -> Result<()> {
        let ast::Expression::If {
            condition,
            consequence,
            alternative,
        } = expression
        else {
            panic!("Expected If expression, got: {:?}", expression);
        };

        self.compile_expression(condition)?;

        // Dummy value, which we will change later
        let jump_not_truthy_pos = self.emit(Instruction::JumpNotTruthy(0));

        self.compile_block_statement(consequence)?;
        if self.bytecode.instructions.last() == Some(&Instruction::Pop) {
            self.bytecode.instructions.pop();
        }

        // Dummy value, which we will change later
        let jump_pos = self.emit(Instruction::Jump(0));

        let after_consequence_pos = self.bytecode.instructions.len() as u16;
        self.bytecode.instructions[jump_not_truthy_pos] =
            Instruction::JumpNotTruthy(after_consequence_pos);

        self.compile_block_statement(alternative)?;
        if self.bytecode.instructions.last() == Some(&Instruction::Pop) {
            self.bytecode.instructions.pop();
        }

        let after_alternative_pos = self.bytecode.instructions.len() as u16;
        self.bytecode.instructions[jump_pos] = Instruction::Jump(after_alternative_pos);

        Ok(())
    }
}
