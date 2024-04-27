mod error;
mod precedence;

use std::rc::Rc;

use crate::{
    ast::{self, InfixOperatorKind, PrefixOperatorKind},
    lexer::Lexer,
    token::Token,
};
use precedence::Precedence;

pub use error::*;

pub fn parse(input: &str) -> Result<ast::Program> {
    let mut parser = Parser::new(Lexer::new(input));
    parser.parse_program()
}

struct Parser<'a> {
    lexer: Lexer<'a>,

    current_token: Option<Token>,
    peek_token: Option<Token>,
}

impl<'a> Parser<'a> {
    fn new(mut lexer: Lexer<'a>) -> Self {
        let current_token = lexer.next();
        let peek_token = lexer.next();

        Parser {
            lexer,
            current_token,
            peek_token,
        }
    }
}

impl Parser<'_> {
    pub fn step(&mut self) {
        self.current_token = self.lexer.next();
        std::mem::swap(&mut self.current_token, &mut self.peek_token);
    }

    pub fn peek_precedence(&self) -> Option<Precedence> {
        self.peek_token.as_ref().map(Precedence::from)
    }

    fn parse_program(&mut self) -> Result<ast::Program> {
        let mut statements = Vec::new();

        while self.current_token.is_some() {
            let stmt = self.parse_statement()?;
            statements.push(stmt);

            self.step();
        }

        Ok(ast::Program { statements })
    }

    fn parse_statement(&mut self) -> Result<ast::Statement> {
        match &self.current_token {
            Some(Token::Let) => self.parse_let_statement(),
            Some(Token::Return) => self.parse_return_statement(),
            _ => {
                let expression = self.parse_expression(Precedence::Lowest)?;

                if self.peek_token == Some(Token::Semicolon) {
                    self.step();
                }

                Ok(ast::Statement::Expression(expression))
            }
        }
    }

    fn parse_let_statement(&mut self) -> Result<ast::Statement> {
        self.step(); // consume `let`
        let name_token = self.current_token.take();
        let Some(Token::Ident(name)) = name_token else {
            return Err(Error::unexpected_token(&name_token));
        };

        if self.peek_token != Some(Token::Assign) {
            return Err(Error::unexpected_token(&self.peek_token));
        }
        self.step();
        self.step();

        let value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Some(Token::Semicolon) {
            self.step();
        }

        Ok(ast::Statement::Let { name, value })
    }

    fn parse_return_statement(&mut self) -> Result<ast::Statement> {
        self.step(); // consume `return`

        let value = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Some(Token::Semicolon) {
            self.step();
        }

        Ok(ast::Statement::Return(value))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<ast::Expression> {
        let mut left = self.parse_prefix()?;

        while self.peek_token != Some(Token::Semicolon)
            && precedence < self.peek_precedence().unwrap_or(Precedence::Lowest)
        {
            if !self.peek_token.as_ref().map_or(false, |t| t.is_infix()) {
                return Ok(left);
            }

            self.step();
            left = self.parse_infix(left)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self) -> Result<ast::Expression> {
        let expr = match &self.current_token {
            Some(Token::Ident(_)) => {
                let name_token = self.current_token.take();
                let Some(Token::Ident(name)) = name_token else {
                    unreachable!();
                };

                ast::Expression::Identifier(name)
            }
            Some(Token::String(_)) => {
                let string_token = self.current_token.take();
                let Some(Token::String(value)) = string_token else {
                    unreachable!();
                };

                ast::Expression::StringLiteral(value)
            }
            Some(Token::Int(value)) => ast::Expression::IntegerLiteral(value.parse()?),
            Some(Token::Bang) | Some(Token::Minus) => self.parse_prefix_operator()?,
            Some(Token::True) => ast::Expression::BooleanLiteral(true),
            Some(Token::False) => ast::Expression::BooleanLiteral(false),
            Some(Token::Lparen) => self.parse_grouped()?,
            Some(Token::LBracket) => self.parse_array_literal()?,
            Some(Token::If) => self.parse_if_expression()?,
            Some(Token::Function) => self.parse_function_literal()?,
            token => return Err(Error::NotAnExpression(token.clone())),
        };

        Ok(expr)
    }

    fn parse_infix(&mut self, left: ast::Expression) -> Result<ast::Expression> {
        let expr = match &self.current_token {
            Some(Token::Plus)
            | Some(Token::Minus)
            | Some(Token::Asterisk)
            | Some(Token::Slash)
            | Some(Token::Eq)
            | Some(Token::NotEq)
            | Some(Token::Lt)
            | Some(Token::Gt) => self.parse_infix_operator(left)?,
            Some(Token::Lparen) => self.parse_call_expression(left)?,
            Some(Token::LBracket) => self.parse_index_expression(left)?,
            _ => left,
        };

        Ok(expr)
    }

    fn parse_prefix_operator(&mut self) -> Result<ast::Expression> {
        let operator = PrefixOperatorKind::try_from(&self.current_token)?;
        self.step();

        Ok(ast::Expression::PrefixOperator {
            operator,
            right: Box::new(self.parse_expression(Precedence::Prefix)?),
        })
    }

    fn parse_grouped(&mut self) -> Result<ast::Expression> {
        self.step();

        let expression = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token == Some(Token::Rparen) {
            self.step();
            Ok(expression)
        } else {
            Err(Error::unexpected_token(&self.peek_token))
        }
    }

    fn parse_if_expression(&mut self) -> Result<ast::Expression> {
        if self.peek_token != Some(Token::Lparen) {
            return Err(Error::unexpected_token(&self.peek_token));
        }
        self.step();
        self.step();

        let condition = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token != Some(Token::Rparen) {
            return Err(Error::unexpected_token(&self.peek_token));
        }
        self.step();

        if self.peek_token != Some(Token::Lsquigly) {
            return Err(Error::unexpected_token(&self.peek_token));
        }
        self.step();

        let consequence = self.parse_block_statement()?;

        if self.current_token != Some(Token::Rsquigly) {
            return Err(Error::unexpected_token(&self.current_token));
        }

        if self.peek_token == Some(Token::Else) {
            self.step();

            if self.peek_token != Some(Token::Lsquigly) {
                return Err(Error::unexpected_token(&self.peek_token));
            }
            self.step();

            let alternative = self.parse_block_statement()?;

            if self.current_token != Some(Token::Rsquigly) {
                return Err(Error::unexpected_token(&self.current_token));
            }

            Ok(ast::Expression::If {
                condition: Box::new(condition),
                consequence,
                alternative,
            })
        } else {
            Ok(ast::Expression::If {
                condition: Box::new(condition),
                consequence,
                alternative: ast::BlockStatement {
                    statements: Rc::new(vec![]),
                },
            })
        }
    }

    fn parse_function_literal(&mut self) -> Result<ast::Expression> {
        if self.peek_token != Some(Token::Lparen) {
            return Err(Error::unexpected_token(&self.peek_token));
        }
        self.step();

        let parameters = self.parse_function_parameters()?;

        if self.peek_token != Some(Token::Lsquigly) {
            return Err(Error::unexpected_token(&self.peek_token));
        }
        self.step();

        let body = self.parse_block_statement()?;

        Ok(ast::Expression::FunctionLiteral { parameters, body })
    }

    fn parse_ident(&mut self) -> Result<String> {
        let name_token = self.current_token.take();
        let Some(Token::Ident(name)) = name_token else {
            return Err(Error::unexpected_token(&name_token));
        };

        Ok(name)
    }

    fn parse_function_parameters(&mut self) -> Result<Vec<String>> {
        let mut identifiers = vec![];

        self.step();
        if self.current_token == Some(Token::Rparen) {
            return Ok(identifiers);
        }

        identifiers.push(self.parse_ident()?);

        while self.peek_token == Some(Token::Comma) {
            self.step();
            self.step();

            identifiers.push(self.parse_ident()?);
        }

        if self.peek_token != Some(Token::Rparen) {
            return Err(Error::unexpected_token(&self.peek_token));
        }
        self.step();

        Ok(identifiers)
    }

    fn parse_block_statement(&mut self) -> Result<ast::BlockStatement> {
        self.step();

        let mut statements = vec![];

        while self.current_token.is_some() && self.current_token != Some(Token::Rsquigly) {
            let stmt = self.parse_statement()?;
            statements.push(stmt);

            self.step();
        }

        Ok(ast::BlockStatement {
            statements: Rc::new(statements),
        })
    }

    fn parse_infix_operator(&mut self, left: ast::Expression) -> Result<ast::Expression> {
        let operator = InfixOperatorKind::try_from(&self.current_token)?;
        let precedence = Precedence::from(&self.current_token);

        self.step();

        let right = self.parse_expression(precedence)?;

        Ok(ast::Expression::InfixOperator {
            operator,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_call_expression(&mut self, left: ast::Expression) -> Result<ast::Expression> {
        Ok(ast::Expression::FunctionCall {
            function: Box::new(left),
            arguments: self.parse_expression_list(Token::Rparen)?,
        })
    }

    fn parse_expression_list(&mut self, end: Token) -> Result<Vec<ast::Expression>> {
        let end_token = Some(end);
        let mut list = vec![];

        self.step();
        if self.current_token == end_token {
            return Ok(list);
        }

        list.push(self.parse_expression(Precedence::Lowest)?);

        while self.peek_token == Some(Token::Comma) {
            self.step();
            self.step();

            list.push(self.parse_expression(Precedence::Lowest)?);
        }

        if self.peek_token != end_token {
            return Err(Error::unexpected_token(&self.peek_token));
        }
        self.step();

        Ok(list)
    }

    fn parse_array_literal(&mut self) -> Result<ast::Expression> {
        let elements = self.parse_expression_list(Token::RBracket)?;
        Ok(ast::Expression::ArrayLiteral(elements))
    }

    fn parse_index_expression(&mut self, left: ast::Expression) -> Result<ast::Expression> {
        self.step();
        let index = self.parse_expression(Precedence::Lowest)?;

        if self.peek_token != Some(Token::RBracket) {
            return Err(Error::unexpected_token(&self.peek_token));
        }
        self.step();

        Ok(ast::Expression::Index {
            left: Box::new(left),
            index: Box::new(index),
        })
    }
}

#[cfg(test)]
mod test {
    use crate::ast;
    use crate::parse::{parse, Result};

    #[test]
    fn test_let_statements() -> Result<()> {
        let tests = [
            ("let x = 5;", "x", "5"),
            ("let y = 10;", "y", "10"),
            ("let foobar = y;", "foobar", "y"),
        ];

        for (input, expected_name, expected_expr) in tests {
            let program = parse(input)?;

            assert_eq!(program.statements.len(), 1);

            let ast::Statement::Let { name, value } = &program.statements[0] else {
                panic!("Expected let statement, got: {:?}", program.statements[0]);
            };

            assert_eq!(name, expected_name);
            assert_eq!(value.debug_str(), expected_expr);
        }

        Ok(())
    }

    #[test]
    fn test_invalid_let_statement() {
        let inputs = ["let x 5;", "let = 10;", "let 838383;"];

        for (index, input) in inputs.iter().enumerate() {
            let program = parse(input);

            assert!(program.is_err(), "test case {} should have failed", index);
        }
    }

    #[test]
    fn test_return_statements() -> Result<()> {
        let tests = [
            ("return 5;", "5"),
            ("return 10;", "10"),
            ("return 5 * y", "(5 * y)"),
        ];

        for (input, expected_expr) in tests {
            let program = parse(input)?;

            assert_eq!(program.statements.len(), 1);

            let ast::Statement::Return(value) = &program.statements[0] else {
                panic!(
                    "Expected return statement, got: {:?}",
                    program.statements[0]
                );
            };

            assert_eq!(value.debug_str(), expected_expr);
        }

        Ok(())
    }

    #[test]
    fn test_identifier_expression() -> Result<()> {
        let input = "foobar;";

        let program = parse(input)?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::Identifier(ref name)) =
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

        let program = parse(input)?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::IntegerLiteral(literal)) =
            program.statements[0]
        else {
            panic!("Expected integer literal, got: {:?}", program.statements[0]);
        };

        assert_eq!(literal, 5);
        Ok(())
    }

    #[test]
    fn test_string_literal_expression() -> Result<()> {
        let input = r#""hello world""#;

        let program = parse(input)?;
        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::StringLiteral(ref literal)) =
            program.statements[0]
        else {
            panic!("Expected string literal, got: {:?}", program.statements[0])
        };

        assert_eq!(literal, "hello world");
        Ok(())
    }

    #[test]
    fn test_boolean_literal_expression() -> Result<()> {
        let tests = [("true;", true), ("false", false)];

        for (input, expected) in tests {
            let program = parse(input)?;

            let ast::Statement::Expression(ast::Expression::BooleanLiteral(literal)) =
                program.statements[0]
            else {
                panic!("Expected boolean literal, got: {:?}", program.statements[0]);
            };

            assert_eq!(literal, expected);
        }

        Ok(())
    }

    #[test]
    fn test_prefix_expressions() -> Result<()> {
        let tests = [
            (
                "!5;",
                ast::Expression::PrefixOperator {
                    operator: ast::PrefixOperatorKind::Not,
                    right: Box::new(ast::Expression::IntegerLiteral(5)),
                },
            ),
            (
                "-15;",
                ast::Expression::PrefixOperator {
                    operator: ast::PrefixOperatorKind::Negative,
                    right: Box::new(ast::Expression::IntegerLiteral(15)),
                },
            ),
            (
                "!false",
                ast::Expression::PrefixOperator {
                    operator: ast::PrefixOperatorKind::Not,
                    right: Box::new(ast::Expression::BooleanLiteral(false)),
                },
            ),
            (
                "!true",
                ast::Expression::PrefixOperator {
                    operator: ast::PrefixOperatorKind::Not,
                    right: Box::new(ast::Expression::BooleanLiteral(true)),
                },
            ),
        ];

        for (input, expected) in tests {
            let program = parse(input)?;

            assert_eq!(program.statements.len(), 1);
            let stmt = &program.statements[0];
            let ast::Statement::Expression(expr) = stmt else {
                panic!("Expected expression, got: {:?}", stmt);
            };

            assert_eq!(*expr, expected);
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

        for (input, expected_left, expected_operator, expected_right) in tests {
            let program = parse(input)?;

            assert_eq!(program.statements.len(), 1);

            let stmt = &program.statements[0];
            let ast::Statement::Expression(ast::Expression::InfixOperator {
                operator,
                left,
                right,
            }) = stmt
            else {
                panic!("Expected infix operator expression, got: {:?}", stmt);
            };

            let ast::Expression::IntegerLiteral(left_val) = **left else {
                panic!(
                    "Expected left expression to be an integer literal, got: {:?}",
                    left
                );
            };

            let ast::Expression::IntegerLiteral(right_val) = **right else {
                panic!(
                    "Expected right expression to be an integer literal, got: {:?}",
                    right
                );
            };

            assert_eq!(*operator, expected_operator);
            assert_eq!(left_val, expected_left);
            assert_eq!(right_val, expected_right);
        }

        Ok(())
    }

    #[test]
    fn test_operator_precedence_parsing() -> Result<()> {
        let tests = [
            ("-a * b", "((-a) * b);"),
            ("!-a", "(!(-a));"),
            ("a + b + c", "((a + b) + c);"),
            ("a + b - c", "((a + b) - c);"),
            ("a * b * c", "((a * b) * c);"),
            ("a * b / c", "((a * b) / c);"),
            ("a + b / c", "(a + (b / c));"),
            ("a + b * c + d / e -f", "(((a + (b * c)) + (d / e)) - f);"),
            ("3 + 4; -5 * 5", "(3 + 4);((-5) * 5);"),
            ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4));"),
            ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4));"),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)));",
            ),
            ("true", "true;"),
            ("false", "false;"),
            ("3 > 5 == false", "((3 > 5) == false);"),
            ("3 < 5 == true", "((3 < 5) == true);"),
            ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4);"),
            ("(5 + 5) * 2", "((5 + 5) * 2);"),
            ("2 / (5 + 5)", "(2 / (5 + 5));"),
            ("-(5 + 5)", "(-(5 + 5));"),
            ("!(true == true)", "(!(true == true));"),
            ("a + add(b*c) + d", "((a + add((b * c))) + d);"),
            (
                "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
                "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)));",
            ),
            (
                "add(a + b + c * d / f + g)",
                "add((((a + b) + ((c * d) / f)) + g));",
            ),
            (
                "a * [1,2,3,4][b*c]*d",
                "((a * ([1, 2, 3, 4][(b * c)])) * d);",
            ),
            (
                "add(a * b[2], b[1], 2 * [1, 2][1])",
                "add((a * (b[2])), (b[1]), (2 * ([1, 2][1])));",
            ),
        ];

        for (input, expected) in tests {
            let program = parse(input)?;

            assert_eq!(program.debug_str(), expected);
        }

        Ok(())
    }

    #[test]
    fn test_if_expression() -> Result<()> {
        let input = "if (x < y) {x}";

        let program = parse(input)?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::If {
            condition,
            consequence,
            alternative,
        }) = &program.statements[0]
        else {
            panic!("Expected if expression, got: {:?}", program.statements[0]);
        };

        assert_eq!(condition.debug_str(), "(x < y)");
        assert_eq!(consequence.statements.len(), 1);
        assert_eq!(alternative.statements.len(), 0);

        let ast::Statement::Expression(ast::Expression::Identifier(ref ident)) =
            consequence.statements[0]
        else {
            panic!(
                "Expected identifier statement, got: {:?}",
                consequence.statements[0]
            );
        };
        assert_eq!(ident, "x");

        Ok(())
    }

    #[test]
    fn test_if_else_expression() -> Result<()> {
        let input = "if (x < y) {x} else {y}";

        let program = parse(input)?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::If {
            condition,
            consequence,
            alternative,
        }) = &program.statements[0]
        else {
            panic!("Expected if expression, got: {:?}", program.statements[0]);
        };

        assert_eq!(condition.debug_str(), "(x < y)");
        assert_eq!(consequence.statements.len(), 1);
        assert_eq!(alternative.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::Identifier(ref ident)) =
            consequence.statements[0]
        else {
            panic!(
                "Expected identifier statement, got: {:?}",
                consequence.statements[0]
            );
        };
        assert_eq!(ident, "x");

        let ast::Statement::Expression(ast::Expression::Identifier(ref ident)) =
            alternative.statements[0]
        else {
            panic!(
                "Expected identifier statement, got: {:?}",
                alternative.statements[0]
            );
        };
        assert_eq!(ident, "y");

        Ok(())
    }

    #[test]
    fn test_function_literal_parsing() -> Result<()> {
        let input = "fn(x, y) {x + y;}";

        let program = parse(input)?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::FunctionLiteral { parameters, body }) =
            &program.statements[0]
        else {
            panic!(
                "Expected function literal, got: {:?}",
                program.statements[0]
            );
        };

        assert_eq!(parameters.len(), 2);
        assert_eq!(parameters[0], "x");
        assert_eq!(parameters[1], "y");

        assert_eq!(body.statements.len(), 1);
        assert_eq!(body.statements[0].debug_str(), "(x + y)");

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
            let program = parse(input)?;

            assert_eq!(program.statements.len(), 1);

            let ast::Statement::Expression(ast::Expression::FunctionLiteral {
                parameters,
                body: _,
            }) = &program.statements[0]
            else {
                panic!(
                    "Expected function literal, got: {:?}",
                    program.statements[0]
                );
            };

            assert_eq!(*parameters, expected);
        }

        Ok(())
    }

    #[test]
    fn test_function_call_expression() -> Result<()> {
        let input = "add(1, 2*3, 4 + 5);";

        let program = parse(input)?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::FunctionCall {
            function,
            arguments,
        }) = &program.statements[0]
        else {
            panic!("Expected call expression, got: {:?}", program.statements[0]);
        };

        assert_eq!(function.debug_str(), "add");
        assert_eq!(arguments.len(), 3);
        assert_eq!(arguments[0].debug_str(), "1");
        assert_eq!(arguments[1].debug_str(), "(2 * 3)");
        assert_eq!(arguments[2].debug_str(), "(4 + 5)");

        Ok(())
    }

    #[test]
    fn test_function_arguments_parsing() -> Result<()> {
        let tests = [
            ("add();", vec![]),
            ("add(1);", vec!["1"]),
            ("add(1, 2 * 3, 4 + 5);", vec!["1", "(2 * 3)", "(4 + 5)"]),
        ];

        for (input, expected) in tests {
            let program = parse(input)?;

            assert_eq!(program.statements.len(), 1);

            let ast::Statement::Expression(ast::Expression::FunctionCall {
                function: _,
                arguments,
            }) = &program.statements[0]
            else {
                panic!(
                    "Expected function call expression, got: {:?}",
                    program.statements[0]
                );
            };

            assert_eq!(
                arguments
                    .iter()
                    .map(|arg| arg.debug_str())
                    .collect::<Vec<String>>(),
                expected
            );
        }

        Ok(())
    }

    #[test]
    fn test_parse_array_literal() -> Result<()> {
        let input = "[1, 2 * 2, 3 + 3]";

        let program = parse(input)?;
        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::ArrayLiteral(ref arr)) =
            program.statements[0]
        else {
            panic!("Expected array literal, got {:?}", program.statements[0])
        };

        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].debug_str(), "1");
        assert_eq!(arr[1].debug_str(), "(2 * 2)");
        assert_eq!(arr[2].debug_str(), "(3 + 3)");

        Ok(())
    }

    #[test]
    fn test_parse_index_expression() -> Result<()> {
        let input = "myArray[1 + 1]";

        let program = parse(input)?;
        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::Index { left, index }) =
            &program.statements[0]
        else {
            panic!("Expected index expression, got {:?}", program.statements[0])
        };

        assert_eq!(**left, ast::Expression::Identifier("myArray".to_string()));
        assert_eq!(
            **index,
            ast::Expression::InfixOperator {
                operator: ast::InfixOperatorKind::Add,
                left: Box::new(ast::Expression::IntegerLiteral(1)),
                right: Box::new(ast::Expression::IntegerLiteral(1))
            }
        );

        Ok(())
    }
}
