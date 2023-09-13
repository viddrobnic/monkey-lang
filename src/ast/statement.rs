use crate::{
    evaluate::{self, Environment, Evaluate},
    object::Object,
    parse::{Error, Parse, Precedence},
    token::Token,
};

use super::{Expression, Identifier};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Let(Let),
    Return(Return),
    Expression(Expression),
}

impl Parse for Statement {
    fn parse(
        parser: &mut crate::parse::Parser,
        precedence: crate::parse::Precedence,
        _: Option<Expression>,
    ) -> crate::parse::Result<Self> {
        let stmt = match parser.get_current_token() {
            Token::Let => Self::Let(Let::parse(parser, precedence, None)?),
            Token::Return => Self::Return(Return::parse(parser, precedence, None)?),
            _ => {
                let expression = Expression::parse(parser, precedence, None)?;

                if *parser.get_peek_token() == Token::Semicolon {
                    parser.step();
                }

                Self::Expression(expression)
            }
        };

        Ok(stmt)
    }
}

impl Evaluate for Statement {
    fn evaluate(&self, environment: &mut Environment) -> evaluate::Result<Object> {
        match self {
            Statement::Let(stmt) => stmt.evaluate(environment),
            Statement::Return(return_expr) => Ok(Object::Return(Box::new(
                return_expr.value.evaluate(environment)?,
            ))),
            Statement::Expression(expr) => expr.evaluate(environment),
        }
    }
}

impl Statement {
    pub fn debug_str(&self) -> String {
        match self {
            Self::Let(let_stmt) => format!(
                "let {} = {};",
                let_stmt.name.name,
                let_stmt.value.debug_str()
            ),
            Self::Return(return_stmt) => format!("return {};", return_stmt.value.debug_str()),
            Self::Expression(expr) => expr.debug_str(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Let {
    pub name: Identifier,
    pub value: Expression,
}

impl Parse for Let {
    fn parse(
        parser: &mut crate::parse::Parser,
        _: crate::parse::Precedence,
        _: Option<Expression>,
    ) -> crate::parse::Result<Self> {
        let name = match parser.get_peek_token() {
            Token::Ident(name) => name.clone(),
            token => return Err(Error::UnexpectedToken(token.clone())),
        };
        parser.step();

        if *parser.get_peek_token() != Token::Assign {
            return Err(Error::UnexpectedToken(parser.get_peek_token().clone()));
        }
        parser.step();
        parser.step();

        let value = Expression::parse(parser, Precedence::Lowest, None)?;

        if *parser.get_peek_token() == Token::Semicolon {
            parser.step();
        }

        Ok(Self {
            name: Identifier { name },
            value,
        })
    }
}

impl Evaluate for Let {
    fn evaluate(&self, environment: &mut Environment) -> evaluate::Result<Object> {
        let val = self.value.evaluate(environment)?;
        environment.set(self.name.name.clone(), val);

        Ok(Object::Null)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub value: Expression,
}

impl Parse for Return {
    fn parse(
        parser: &mut crate::parse::Parser,
        precedence: crate::parse::Precedence,
        _: Option<Expression>,
    ) -> crate::parse::Result<Self> {
        parser.step();

        let value = Expression::parse(parser, precedence, None)?;

        if *parser.get_peek_token() == Token::Semicolon {
            parser.step();
        }

        Ok(Self { value })
    }
}
