use std::mem;

use crate::{ast, lexer::Lexer, token::Token};

pub mod error;
use error::*;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum Precedence {
    Lowest,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
}

impl From<&Token> for Precedence {
    fn from(value: &Token) -> Self {
        match value {
            Token::Eq | Token::NotEq => Self::Equals,
            Token::Lt | Token::Gt => Self::LessGreater,
            Token::Plus | Token::Minus => Self::Sum,
            Token::Slash | Token::Asterisk => Self::Product,
            _ => Self::Lowest,
        }
    }
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
            Token::Return => self.parse_return_statement(),
            _ => self.parse_expression_statement(),
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

        let let_statement = ast::Let {
            name: ast::Identifier { name },
            value: ast::Expression::Empty,
        };

        Ok(ast::Statement::Let(let_statement))
    }

    fn parse_return_statement(&mut self) -> Result<ast::Statement> {
        self.next_token();

        // TODO: Parse the expression. Currently we are skipping it
        // until we encounter a semicolon.
        while self.current_token != Token::Semicolon {
            self.next_token();
        }

        let return_statement = ast::Return {
            value: ast::Expression::Empty,
        };

        Ok(ast::Statement::Return(return_statement))
    }

    fn parse_expression_statement(&mut self) -> Result<ast::Statement> {
        let expression = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Token::Semicolon {
            self.next_token();
        }

        Ok(ast::Statement::Expression(expression))
    }

    fn peek_precedence(&self) -> Precedence {
        Precedence::from(&self.peek_token)
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<ast::Expression> {
        let mut left = self.parse_prefix()?;

        while self.peek_token != Token::Semicolon && precedence < self.peek_precedence() {
            if !self.peek_token.is_infix() {
                return Ok(left);
            }

            self.next_token();
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<ast::Expression> {
        match &self.current_token {
            Token::Ident(name) => Ok(ast::Expression::Identifier(ast::Identifier {
                name: name.clone(),
            })),
            Token::Int(value) => Ok(ast::Expression::IntegerLiteral(ast::IntegerLiteral {
                value: value.parse()?,
            })),
            Token::Bang | Token::Minus => self.parse_prefix_expression(),
            Token::True => Ok(ast::Expression::BooleanLiteral(ast::BooleanLiteral {
                value: true,
            })),
            Token::False => Ok(ast::Expression::BooleanLiteral(ast::BooleanLiteral {
                value: false,
            })),
            Token::Lparen => self.parse_grouped_expression(),
            Token::If => self.parse_if_expression(),
            Token::Function => self.parse_function_literal(),
            token => Err(Error::NotAnExpression(token.clone())),
        }
    }

    fn parse_prefix_expression(&mut self) -> Result<ast::Expression> {
        let operator = ast::PrefixOperatorKind::try_from(&self.current_token)?;
        self.next_token();

        let expression = ast::PrefixOperator {
            operator,
            right: Box::new(self.parse_expression(Precedence::Prefix)?),
        };
        Ok(ast::Expression::PrefixOperator(expression))
    }

    fn parse_grouped_expression(&mut self) -> Result<ast::Expression> {
        self.next_token();

        let expression = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Token::Rparen {
            self.next_token();
            Ok(expression)
        } else {
            Err(Error::UnexpectedToken(self.peek_token.clone()))
        }
    }

    fn parse_if_expression(&mut self) -> Result<ast::Expression> {
        if self.peek_token != Token::Lparen {
            return Err(Error::UnexpectedToken(self.peek_token.clone()));
        }
        self.next_token();
        self.next_token();

        let condition = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token != Token::Rparen {
            return Err(Error::UnexpectedToken(self.peek_token.clone()));
        }
        self.next_token();

        if self.peek_token != Token::Lsquigly {
            return Err(Error::UnexpectedToken(self.peek_token.clone()));
        }
        self.next_token();

        let consequence = self.parse_block_statement()?;
        if self.current_token != Token::Rsquigly {
            return Err(Error::UnexpectedToken(self.current_token.clone()));
        }

        let mut expr = ast::IfExpression {
            condition: Box::new(condition),
            consequence,
            alternative: ast::BlockStatement { statements: vec![] },
        };

        if self.peek_token == Token::Else {
            self.next_token();

            if self.peek_token != Token::Lsquigly {
                return Err(Error::UnexpectedToken(self.peek_token.clone()));
            }
            self.next_token();

            expr.alternative = self.parse_block_statement()?;

            if self.current_token != Token::Rsquigly {
                return Err(Error::UnexpectedToken(self.current_token.clone()));
            }
        }

        Ok(ast::Expression::If(expr))
    }

    fn parse_function_literal(&mut self) -> Result<ast::Expression> {
        if self.peek_token != Token::Lparen {
            return Err(Error::UnexpectedToken(self.peek_token.clone()));
        }
        self.next_token();

        let parameters = self.parse_function_parameters()?;

        if self.peek_token != Token::Lsquigly {
            return Err(Error::UnexpectedToken(self.peek_token.clone()));
        }
        self.next_token();

        let body = self.parse_block_statement()?;

        Ok(ast::Expression::FunctionLiteral(ast::FunctionLiteral {
            parameters,
            body,
        }))
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<ast::Identifier>> {
        let mut identifiers = vec![];

        self.next_token();
        if self.current_token == Token::Rparen {
            return Ok(identifiers);
        }

        identifiers.push(token_to_ident(&self.current_token)?);

        while self.peek_token == Token::Comma {
            self.next_token();
            self.next_token();

            identifiers.push(token_to_ident(&self.current_token)?);
        }

        if self.peek_token != Token::Rparen {
            return Err(Error::UnexpectedToken(self.peek_token.clone()));
        }
        self.next_token();

        Ok(identifiers)
    }

    fn parse_block_statement(&mut self) -> Result<ast::BlockStatement> {
        self.next_token();

        let mut block = ast::BlockStatement { statements: vec![] };

        while self.current_token != Token::Rsquigly && self.current_token != Token::Eof {
            let stmt = self.parse_statement()?;
            block.statements.push(stmt);

            self.next_token();
        }

        Ok(block)
    }

    fn parse_infix(&mut self, left: ast::Expression) -> Result<ast::Expression> {
        let operator = ast::InfixOperatorKind::try_from(&self.current_token)?;
        let precedence = Precedence::from(&self.current_token);

        self.next_token();

        let right = self.parse_expression(precedence)?;

        let expression = ast::InfixOperator {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        };

        Ok(ast::Expression::InfixOperator(expression))
    }
}

fn token_to_ident(token: &Token) -> Result<ast::Identifier> {
    let Token::Ident(ident) = token else {
        return Err(Error::UnexpectedToken(token.clone()));
    };
    Ok(ast::Identifier {
        name: ident.clone(),
    })
}

#[cfg(test)]
mod tests {
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
    fn test_invalid_let_statement() {
        let inputs = ["let x 5;", "let = 10;", "let 838383;"];

        for (index, input) in inputs.iter().enumerate() {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program();

            assert!(program.is_err(), "test case {} should have failed", index);
        }
    }

    #[test]
    fn test_return_statements() -> Result<()> {
        let input = r#"
            return 5;
            return 10;
            return 993232;
        "#;

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program()?;

        assert_eq!(program.statements.len(), 3);

        for stmt in program.statements {
            assert!(
                matches!(stmt, ast::Statement::Return(_)),
                "Expected return statement, got: {:?}",
                stmt
            );
        }

        Ok(())
    }

    #[test]
    fn test_identifier_expression() -> Result<()> {
        let input = "foobar;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program()?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::Identifier(ast::Identifier { ref name })) =
            program.statements[0]
        else {
            panic!(
                "Expected identifier expression statement, got: {:?}",
                program.statements[0]
            );
        };

        assert_eq!(name, "foobar");
        Ok(())
    }

    #[test]
    fn test_integer_literal_expression() -> Result<()> {
        let input = "5;";

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program()?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::IntegerLiteral(ref literal)) =
            program.statements[0]
        else {
            panic!("Expected integer literal, got: {:?}", program.statements[0]);
        };

        assert_eq!(literal.value, 5);
        Ok(())
    }

    #[test]
    fn test_boolean_literal_expression() -> Result<()> {
        let tests = [("true;", true), ("false", false)];

        for (input, expected) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program()?;

            let ast::Statement::Expression(ast::Expression::BooleanLiteral(ref literal)) =
                program.statements[0]
            else {
                panic!("Expected boolean literal, got: {:?}", program.statements[0]);
            };

            assert_eq!(literal.value, expected);
        }

        Ok(())
    }

    #[test]
    fn test_prefix_expressions() -> Result<()> {
        let tests = [
            (
                "!5;",
                ast::PrefixOperator {
                    operator: ast::PrefixOperatorKind::Not,
                    right: Box::new(ast::Expression::IntegerLiteral(ast::IntegerLiteral {
                        value: 5,
                    })),
                },
            ),
            (
                "-15;",
                ast::PrefixOperator {
                    operator: ast::PrefixOperatorKind::Negative,
                    right: Box::new(ast::Expression::IntegerLiteral(ast::IntegerLiteral {
                        value: 15,
                    })),
                },
            ),
            (
                "!false",
                ast::PrefixOperator {
                    operator: ast::PrefixOperatorKind::Not,
                    right: Box::new(ast::Expression::BooleanLiteral(ast::BooleanLiteral {
                        value: false,
                    })),
                },
            ),
            (
                "!true",
                ast::PrefixOperator {
                    operator: ast::PrefixOperatorKind::Not,
                    right: Box::new(ast::Expression::BooleanLiteral(ast::BooleanLiteral {
                        value: true,
                    })),
                },
            ),
        ];

        for (input, expected) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program()?;

            assert_eq!(program.statements.len(), 1);
            let stmt = &program.statements[0];
            let ast::Statement::Expression(ast::Expression::PrefixOperator(prefix)) = stmt else {
                panic!("Expected prefix operator expression, got: {:?}", stmt);
            };

            assert_eq!(*prefix, expected);
        }

        Ok(())
    }

    #[test]
    fn test_infix_expressions() -> Result<()> {
        let tests = [
            ("5 + 5;", 5, ast::InfixOperatorKind::Add, 5),
            ("5 - 5", 5, ast::InfixOperatorKind::Subtract, 5),
            ("5 * 5", 5, ast::InfixOperatorKind::Multiply, 5),
            ("5 / 5", 5, ast::InfixOperatorKind::Divide, 5),
            ("5 == 5", 5, ast::InfixOperatorKind::Equal, 5),
            ("5 != 5", 5, ast::InfixOperatorKind::NotEqual, 5),
            ("5 > 5", 5, ast::InfixOperatorKind::GreaterThan, 5),
            ("5 < 5", 5, ast::InfixOperatorKind::LessThan, 5),
        ];

        for (input, left, operator, right) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program()?;

            assert_eq!(program.statements.len(), 1);

            let stmt = &program.statements[0];
            let ast::Statement::Expression(ast::Expression::InfixOperator(infix)) = stmt else {
                panic!("Expected infix operator expression, got: {:?}", stmt);
            };

            let ast::Expression::IntegerLiteral(ast::IntegerLiteral { value: left_val }) =
                *infix.left
            else {
                panic!(
                    "Expected left expression to be an integer literal, got: {:?}",
                    infix.left
                );
            };

            let ast::Expression::IntegerLiteral(ast::IntegerLiteral { value: right_val }) =
                *infix.right
            else {
                panic!(
                    "Expected right expression to be an integer literal, got: {:?}",
                    infix.right
                );
            };

            assert_eq!(infix.operator, operator);
            assert_eq!(left_val, left);
            assert_eq!(right_val, right);
        }

        Ok(())
    }

    #[test]
    fn test_operator_precedence_parsing() -> Result<()> {
        let tests = [
            ("-a * b", "((-a) * b)"),
            ("!-a", "(!(-a))"),
            ("a + b + c", "((a + b) + c)"),
            ("a + b - c", "((a + b) - c)"),
            ("a * b * c", "((a * b) * c)"),
            ("a * b / c", "((a * b) / c)"),
            ("a + b / c", "(a + (b / c))"),
            ("a + b * c + d / e -f", "(((a + (b * c)) + (d / e)) - f)"),
            ("3 + 4; -5 * 5", "(3 + 4)((-5) * 5)"),
            ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4))"),
            ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4))"),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            ),
            ("true", "true"),
            ("false", "false"),
            ("3 > 5 == false", "((3 > 5) == false)"),
            ("3 < 5 == true", "((3 < 5) == true)"),
            ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4)"),
            ("(5 + 5) * 2", "((5 + 5) * 2)"),
            ("2 / (5 + 5)", "(2 / (5 + 5))"),
            ("-(5 + 5)", "(-(5 + 5))"),
            ("!(true == true)", "(!(true == true))"),
        ];

        for (input, expected) in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);

            let program = parser.parse_program()?;

            assert_eq!(program.debug_str(), expected);
        }

        Ok(())
    }

    #[test]
    fn test_if_expression() -> Result<()> {
        let input = "if (x < y) {x}";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program()?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::If(ref if_stmt)) = program.statements[0]
        else {
            panic!("Expected if expression, got: {:?}", program.statements[0]);
        };

        assert_eq!(if_stmt.condition.debug_str(), "(x < y)");
        assert_eq!(if_stmt.consequence.statements.len(), 1);
        assert_eq!(if_stmt.alternative.statements.len(), 0);

        let ast::Statement::Expression(ast::Expression::Identifier(ref ident)) =
            if_stmt.consequence.statements[0]
        else {
            panic!(
                "Expected identifier statement, got: {:?}",
                if_stmt.consequence.statements[0]
            );
        };
        assert_eq!(ident.name, "x");

        Ok(())
    }

    #[test]
    fn test_if_else_expression() -> Result<()> {
        let input = "if (x < y) {x} else {y}";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program()?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::If(ref if_stmt)) = program.statements[0]
        else {
            panic!("Expected if expression, got: {:?}", program.statements[0]);
        };

        assert_eq!(if_stmt.condition.debug_str(), "(x < y)");
        assert_eq!(if_stmt.consequence.statements.len(), 1);
        assert_eq!(if_stmt.alternative.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::Identifier(ref ident)) =
            if_stmt.consequence.statements[0]
        else {
            panic!(
                "Expected identifier statement, got: {:?}",
                if_stmt.consequence.statements[0]
            );
        };
        assert_eq!(ident.name, "x");

        let ast::Statement::Expression(ast::Expression::Identifier(ref ident)) =
            if_stmt.alternative.statements[0]
        else {
            panic!(
                "Expected identifier statement, got: {:?}",
                if_stmt.alternative.statements[0]
            );
        };
        assert_eq!(ident.name, "y");

        Ok(())
    }

    #[test]
    fn test_function_literal_parsing() -> Result<()> {
        let input = "fn(x, y) {x + y;}";

        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program()?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::FunctionLiteral(ref stmt)) =
            program.statements[0]
        else {
            panic!(
                "Expected function literal, got: {:?}",
                program.statements[0]
            );
        };

        assert_eq!(stmt.parameters.len(), 2);
        assert_eq!(stmt.parameters[0].name, "x");
        assert_eq!(stmt.parameters[1].name, "y");

        assert_eq!(stmt.body.statements.len(), 1);
        assert_eq!(stmt.body.statements[0].debug_str(), "(x + y)");

        Ok(())
    }

    #[test]
    fn test_function_parameters_parsing() -> Result<()> {
        let tests = [
            ("fn() {}", vec![]),
            ("fn(x) {}", vec!["x"]),
            ("fn(x, y, z) {}", vec!["x", "y", "z"]),
        ];

        for (input, expected) in tests {
            let mut parser = Parser::new(Lexer::new(input));
            let program = parser.parse_program()?;

            assert_eq!(program.statements.len(), 1);

            let ast::Statement::Expression(ast::Expression::FunctionLiteral(ref stmt)) =
                program.statements[0]
            else {
                panic!(
                    "Expected function literal, got: {:?}",
                    program.statements[0]
                );
            };

            assert_eq!(
                stmt.parameters
                    .iter()
                    .map(|ident| ident.name.to_owned())
                    .collect::<Vec<String>>(),
                expected
            );
        }

        Ok(())
    }

    fn test_let_statement(stmt: &ast::Statement, name: &str) {
        let ast::Statement::Let(let_stmt) = stmt else {
            panic!("statement is not a let statement");
        };

        assert_eq!(let_stmt.name.name, name);
    }
}
