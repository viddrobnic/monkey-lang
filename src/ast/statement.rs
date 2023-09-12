use crate::{
    evaluate::Evaluate,
    object::Object,
    parse::{Error, Parse, Precedence},
    token::Token,
};

use super::{Expression, Identifier};

#[derive(Debug, PartialEq)]
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
    fn evaluate(&self) -> Object {
        match self {
            Statement::Let(_) => todo!(),
            Statement::Return(_) => todo!(),
            Statement::Expression(expr) => expr.evaluate(),
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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
