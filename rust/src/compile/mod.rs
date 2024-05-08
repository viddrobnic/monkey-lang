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

use self::symbol_table::{SymbolScope, SymbolTable};

pub use error::*;

/// Compiles AST to the bytecode.
pub struct Compiler {
    constants: Vec<Object>,

    symbol_table: SymbolTable,

    scopes: Vec<Vec<Instruction>>,
    scope_index: usize,
}

impl Compiler {
    /// Creates a new compiler with empty state
    /// (globals, constants, ...).
    pub fn new() -> Self {
        Self {
            constants: vec![],
            symbol_table: SymbolTable::new(),
            scopes: vec![vec![]],
            scope_index: 0,
        }
    }

    /// Compiles a program.
    ///
    /// The instructions part of the bytecode is overriden,
    /// but the state (globals, constants, ...) is left unchanged.
    /// If you don't want to keep the state between compilations,
    /// initialize a new compiler.
    pub fn compile(&mut self, program: &ast::Program) -> Result<Bytecode> {
        self.scopes = vec![vec![]];
        self.scope_index = 0;

        for stmt in &program.statements {
            self.compile_statement(stmt)?;
        }

        // There should only be one scope if compiler works correctly
        let instructions = self.scopes.pop().expect("Invalid number of scopes!");
        Ok(Bytecode {
            instructions: Rc::new(instructions),
            constants: &self.constants,
        })
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    fn add_constant(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    fn current_instructions(&mut self) -> &mut Vec<Instruction> {
        &mut self.scopes[self.scope_index]
    }

    fn emit(&mut self, instruction: Instruction) -> usize {
        self.current_instructions().push(instruction);
        self.current_instructions().len() - 1
    }

    fn enter_scope(&mut self) {
        self.scopes.push(vec![]);
        self.scope_index += 1;

        self.symbol_table.enclose();
    }

    fn leave_scope(&mut self) -> Vec<Instruction> {
        self.scope_index -= 1;
        let instructions = self.scopes.pop().unwrap_or_default();

        self.symbol_table.leave();

        instructions
    }

    fn compile_statement(&mut self, statement: &ast::Statement) -> Result<()> {
        match statement {
            ast::Statement::Let { name, value } => {
                let symbol = self.symbol_table.define(name.clone());

                self.compile_expression(value)?;
                match symbol.scope {
                    SymbolScope::Global => self.emit(Instruction::SetGlobal(symbol.index)),
                    SymbolScope::Local => self.emit(Instruction::SetLocal(symbol.index as u8)),
                };
            }
            ast::Statement::Return(expr) => {
                self.compile_expression(expr)?;
                self.emit(Instruction::ReturnValue);
            }
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
                        match symbol.scope {
                            SymbolScope::Global => self.emit(Instruction::GetGlobal(symbol.index)),
                            SymbolScope::Local => {
                                self.emit(Instruction::GetLocal(symbol.index as u8))
                            }
                        };
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
            ast::Expression::FunctionLiteral { .. } => self.compile_function_literal(expression)?,
            ast::Expression::FunctionCall {
                function,
                arguments: _,
            } => {
                self.compile_expression(function)?;
                self.emit(Instruction::Call);
            }
            ast::Expression::Index { left, index } => {
                self.compile_expression(left)?;
                self.compile_expression(index)?;

                self.emit(Instruction::Index);
            }
        }

        Ok(())
    }

    fn compile_block_statement(&mut self, statement: &ast::BlockStatement) -> Result<()> {
        if statement.statements.len() == 0 {
            self.emit(Instruction::Null);
            self.emit(Instruction::Pop);
            return Ok(());
        }

        for stmt in statement.statements.iter() {
            self.compile_statement(stmt)?;
        }

        if matches!(
            statement.statements.last(),
            Some(ast::Statement::Let { .. })
        ) {
            self.emit(Instruction::Null);
            self.emit(Instruction::Pop);
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
        if self.current_instructions().last() == Some(&Instruction::Pop) {
            self.current_instructions().pop();
        }

        // Dummy value, which we will change later
        let jump_pos = self.emit(Instruction::Jump(0));

        let after_consequence_pos = self.current_instructions().len() as u16;
        self.current_instructions()[jump_not_truthy_pos] =
            Instruction::JumpNotTruthy(after_consequence_pos);

        self.compile_block_statement(alternative)?;
        if self.current_instructions().last() == Some(&Instruction::Pop) {
            self.current_instructions().pop();
        }

        let after_alternative_pos = self.current_instructions().len() as u16;
        self.current_instructions()[jump_pos] = Instruction::Jump(after_alternative_pos);

        Ok(())
    }

    fn compile_function_literal(&mut self, expression: &ast::Expression) -> Result<()> {
        let ast::Expression::FunctionLiteral {
            parameters: _,
            body,
        } = expression
        else {
            panic!("Expected FunctionLiteral, got: {:?}", expression);
        };

        self.enter_scope();

        self.compile_block_statement(body)?;
        if self.current_instructions().last() == Some(&Instruction::Pop) {
            let idx = self.current_instructions().len() - 1;
            self.current_instructions()[idx] = Instruction::ReturnValue;
        }

        let instructions = self.leave_scope();
        let compiled_fn = Object::CompiledFunction(Rc::new(instructions));

        let constant_idx = self.add_constant(compiled_fn);
        self.emit(Instruction::Constant(constant_idx as u16));

        Ok(())
    }
}
