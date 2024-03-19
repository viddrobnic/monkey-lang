use crate::{
    ast,
    lexer::Lexer,
    parse::{Parser, Result},
};

#[test]
fn test_let_statements() -> Result<()> {
    let tests = [
        ("let x = 5;", "x", "5"),
        ("let y = 10;", "y", "10"),
        ("let foobar = y;", "foobar", "y"),
    ];

    for (input, expected_name, expected_expr) in tests {
        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program()?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Let(ast::Let { name, value }) = &program.statements[0] else {
            panic!("Expected let statement, got: {:?}", program.statements[0]);
        };

        assert_eq!(name.name, expected_name);
        assert_eq!(value.debug_str(), expected_expr);
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
    let tests = [
        ("return 5;", "5"),
        ("return 10;", "10"),
        ("return 5 * y", "(5 * y)"),
    ];

    for (input, expected_expr) in tests {
        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program()?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Return(ast::Return { value }) = &program.statements[0] else {
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
fn test_string_literal_expression() -> Result<()> {
    let input = "\"hello world\";";

    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);

    let program = parser.parse_program()?;

    assert_eq!(program.statements.len(), 1);

    let ast::Statement::Expression(ast::Expression::StringLiteral(ref literal)) =
        program.statements[0]
    else {
        panic!("Expected string literal, got: {:?}", program.statements[0]);
    };

    assert_eq!(literal, "hello world");
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

        let ast::Expression::IntegerLiteral(ast::IntegerLiteral { value: left_val }) = *infix.left
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
        ("a + add(b*c) + d", "((a + add((b * c))) + d)"),
        (
            "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
            "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
        ),
        (
            "add(a + b + c * d / f + g)",
            "add((((a + b) + ((c * d) / f)) + g))",
        ),
        (
            "a * [1, 2, 3, 4][b * c] * d",
            "((a * ([1, 2, 3, 4][(b * c)])) * d)",
        ),
        (
            "add(a * b[2], b[1], 2 * [1, 2][1])",
            "add((a * (b[2])), (b[1]), (2 * ([1, 2][1])))",
        ),
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

    let ast::Statement::Expression(ast::Expression::If(ref if_stmt)) = program.statements[0] else {
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

    let ast::Statement::Expression(ast::Expression::If(ref if_stmt)) = program.statements[0] else {
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

#[test]
fn test_function_call_expression() -> Result<()> {
    let input = "add(1, 2*3, 4 + 5);";

    let mut parser = Parser::new(Lexer::new(input));
    let program = parser.parse_program()?;

    assert_eq!(program.statements.len(), 1);

    let ast::Statement::Expression(ast::Expression::FunctionCall(ref stmt)) = program.statements[0]
    else {
        panic!("Expected call expression, got: {:?}", program.statements[0]);
    };

    assert_eq!(stmt.function.debug_str(), "add");
    assert_eq!(stmt.arguments.len(), 3);
    assert_eq!(stmt.arguments[0].debug_str(), "1");
    assert_eq!(stmt.arguments[1].debug_str(), "(2 * 3)");
    assert_eq!(stmt.arguments[2].debug_str(), "(4 + 5)");

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
        let mut parser = Parser::new(Lexer::new(input));
        let program = parser.parse_program()?;

        assert_eq!(program.statements.len(), 1);

        let ast::Statement::Expression(ast::Expression::FunctionCall(ref stmt)) =
            program.statements[0]
        else {
            panic!(
                "Expected function call expression, got: {:?}",
                program.statements[0]
            );
        };

        assert_eq!(
            stmt.arguments
                .iter()
                .map(|arg| arg.debug_str())
                .collect::<Vec<String>>(),
            expected
        );
    }

    Ok(())
}

#[test]
fn test_array_literal_parsing() -> Result<()> {
    let input = "[1, 2 * 2, 3 + 3]";

    let mut parser = Parser::new(Lexer::new(input));
    let program = parser.parse_program()?;

    assert_eq!(program.statements.len(), 1);

    let ast::Statement::Expression(ast::Expression::ArrayLiteral(ref stmt)) = program.statements[0]
    else {
        panic!("Expected array literal, got: {:?}", program.statements[0]);
    };

    assert_eq!(stmt.elements.len(), 3);
    assert_eq!(stmt.elements[0].debug_str(), "1");
    assert_eq!(stmt.elements[1].debug_str(), "(2 * 2)");
    assert_eq!(stmt.elements[2].debug_str(), "(3 + 3)");

    Ok(())
}

#[test]
fn test_index_expression_parsing() -> Result<()> {
    let input = "myArray[1 + 1]";

    let mut parser = Parser::new(Lexer::new(input));
    let program = parser.parse_program()?;

    assert_eq!(program.statements.len(), 1);

    let ast::Statement::Expression(ast::Expression::Index(ref stmt)) = program.statements[0] else {
        panic!(
            "Expected index expression, got: {:?}",
            program.statements[0]
        );
    };

    assert_eq!(stmt.left.debug_str(), "myArray");
    assert_eq!(stmt.index.debug_str(), "(1 + 1)");

    Ok(())
}
