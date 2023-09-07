use std::mem;

use crate::{ast, lexer::Lexer, token::Token};

pub mod error;
use error::*;

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

    fn next_token(&mut self) {
        self.current_token = self.lexer.next_token();
        mem::swap(&mut self.current_token, &mut self.peek_token);
    }

    pub fn parse_program(&mut self) -> Result<ast::Program> {
        let mut program = ast::Program { statements: vec![] };

        while self.current_token != Token::Eof {
            let stmt = self.parse_statement()?;
            program.statements.push(stmt);

            self.next_token();
        }

        Ok(program)
    }

    fn parse_statement(&mut self) -> Result<ast::Statement> {
        match &self.current_token {
            Token::Let => self.parse_let_statement(),
            token => Err(Error::NotAStatement(token.clone())),
        }
    }

    fn parse_let_statement(&mut self) -> Result<ast::Statement> {
        let name = match &self.peek_token {
            Token::Ident(name) => name.clone(),
            token => return Err(Error::UnexpectedToken(token.clone())),
        };
        self.next_token();

        if self.peek_token != Token::Assign {
            return Err(Error::UnexpectedToken(self.peek_token.clone()));
        }
        self.next_token();

        // TODO: Parse the expression. Currently we are skipping it
        // until we encounter a semicolon
        while self.current_token != Token::Semicolon {
            self.next_token();
        }

        let let_statement = ast::LetStatement {
            name: ast::Identifier { name },
            value: ast::Expression::Empty,
        };

        Ok(ast::Statement::Let(let_statement))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::error::Error;
    use crate::token::Token;
    use crate::{ast, lexer::Lexer};

    use super::error::Result;
    use super::Parser;

    #[test]
    fn test_let_statements() -> Result<()> {
        let input = r#"
            let x = 5;
            let y = 10;
            let foobar = 838383;
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program()?;

        assert_eq!(program.statements.len(), 3);

        let names = ["x", "y", "foobar"];
        for (stmt, name) in program.statements.iter().zip(names) {
            test_let_statement(stmt, name);
        }

        Ok(())
    }

    #[test]
    fn test_invalid_statement() {
        let lexer = Lexer::new("foobar 838383;");
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();

        assert!(matches!(
            program,
            Err(Error::NotAStatement(Token::Ident(_)))
        ));
    }

    #[test]
    fn test_invalid_let_statement() {
        let inputs = ["let x 5;", "let = 10;", "let 838383;"];

        for (index, input) in inputs.iter().enumerate() {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program();

            assert!(program.is_err(), "test case {} should have failed", index);
        }
    }

    fn test_let_statement(stmt: &ast::Statement, name: &str) {
        let ast::Statement::Let(let_stmt) = stmt else {
            panic!("statement is not a let statement");
        };

        assert_eq!(let_stmt.name.name, name);
    }
}
