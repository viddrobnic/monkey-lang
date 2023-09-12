mod expression;
mod operator;
mod statement;

#[cfg(test)]
mod evaluate_tests;
#[cfg(test)]
mod parse_tests;

pub use expression::*;
pub use operator::*;
pub use statement::*;

use crate::{
    evaluate::{self, Evaluate},
    object::Object,
    parse::{self, Parse},
    token::Token,
};

#[derive(Debug, PartialEq)]
pub struct AST {
    pub statements: Vec<Statement>,
}

impl AST {
    pub fn debug_str(&self) -> String {
        let mut res = String::new();
        for stmt in &self.statements {
            res += &stmt.debug_str();
        }
        res
    }
}

impl Parse for AST {
    fn parse(
        parser: &mut crate::parse::Parser,
        precedence: crate::parse::Precedence,
        _: Option<Expression>,
    ) -> parse::Result<Self> {
        let mut statements = Vec::new();

        while *parser.get_current_token() != Token::Eof {
            let stmt = Statement::parse(parser, precedence, None)?;
            statements.push(stmt);

            parser.step();
        }

        Ok(AST { statements })
    }
}

impl Evaluate for AST {
    fn evaluate(&self) -> evaluate::Result<Object> {
        let mut res = Object::Null;

        for stmt in &self.statements {
            res = stmt.evaluate()?;

            match res {
                Object::Return(obj) => return Ok(*obj),
                _ => (),
            }
        }

        Ok(res)
    }
}
