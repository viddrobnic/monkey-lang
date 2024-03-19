mod error;

use crate::{
    ast::{self, Expression, AST},
    lexer::Lexer,
    token::Token,
};
pub use error::{Error, Result};
use std::mem;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
    Index,
}

impl From<&Token> for Precedence {
    fn from(value: &Token) -> Self {
        match value {
            Token::Eq | Token::NotEq => Self::Equals,
            Token::Lt | Token::Gt => Self::LessGreater,
            Token::Plus | Token::Minus => Self::Sum,
            Token::Slash | Token::Asterisk => Self::Product,
            Token::Lparen => Self::Call,
            Token::LBracket => Self::Index,
            _ => Self::Lowest,
        }
    }
}

pub trait Parse
where
    Self: Sized,
{
    fn parse(parser: &mut Parser, precedence: Precedence, left: Option<Expression>)
        -> Result<Self>;
}

pub struct Parser {
    lexer: Lexer,

    current_token: Token,
    peek_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();

        Parser {
            lexer,
            current_token,
            peek_token,
        }
    }

    pub fn step(&mut self) {
        self.current_token = self.lexer.next_token();
        mem::swap(&mut self.current_token, &mut self.peek_token);
    }

    pub fn get_current_token(&self) -> &Token {
        &self.current_token
    }

    pub fn get_peek_token(&self) -> &Token {
        &self.peek_token
    }

    pub fn parse_program(&mut self) -> Result<ast::AST> {
        AST::parse(self, Precedence::Lowest, None)
    }

    pub fn peek_precedence(&self) -> Precedence {
        Precedence::from(&self.peek_token)
    }
}
