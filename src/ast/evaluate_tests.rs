use crate::{evaluate::Evaluate, lexer::Lexer, object::Object, parse::Parser};

#[test]
fn test_eval_integer() {
    let tests = [
        ("5", 5),
        ("10", 10),
        ("-5", -5),
        ("-10", -10),
        ("5 + 5 + 5 + 5 - 10", 10),
        ("2 * 2 * 2 * 2 * 2", 32),
        ("-50 + 100 + -50", 0),
        ("5 * 2 + 10", 20),
        ("5 + 2 * 10", 25),
        ("20 + 2 * -10", 0),
        ("50 / 2 * 2 + 10", 60),
        ("2 * (5 + 10)", 30),
        ("3 * 3 * 3 + 10", 37),
        ("3 * (3 * 3) + 10", 37),
        ("(5 + 10 * 2 + 15/3) * 2 + -10", 50),
    ];

    for (input, expected) in tests.iter() {
        let mut parser = Parser::new(Lexer::new(input));
        let ast = parser.parse_program().unwrap();

        let evaluated = ast.evaluate();
        assert_eq!(evaluated, Object::Integer(*expected));
    }
}

#[test]
fn test_eval_bool() {
    let tests = [
        ("true", true),
        ("false", false),
        ("1 < 2", true),
        ("1 > 2", false),
        ("1 < 1", false),
        ("1 > 1", false),
        ("1 == 1", true),
        ("1 != 1", false),
        ("1 == 2", false),
        ("1 != 2", true),
        ("true == true", true),
        ("false == false", true),
        ("true == false", false),
        ("true != false", true),
        ("false != true", true),
        ("1 < 2 == true", true),
        ("1 < 2 == false", false),
        ("1 > 2 == true", false),
        ("1 > 2 == false", true),
    ];

    for (input, expected) in tests.iter() {
        let mut parser = Parser::new(Lexer::new(input));
        let ast = parser.parse_program().unwrap();

        let evaluated = ast.evaluate();
        assert_eq!(evaluated, Object::Boolean(*expected));
    }
}

#[test]
fn test_bang_operator() {
    let tests = [
        ("!true", false),
        ("!false", true),
        ("!5", false),
        ("!!true", true),
        ("!!false", false),
        ("!!5", true),
    ];

    for (input, expected) in tests.iter() {
        let mut parser = Parser::new(Lexer::new(input));
        let ast = parser.parse_program().unwrap();

        let evaluated = ast.evaluate();
        assert_eq!(evaluated, Object::Boolean(*expected));
    }
}

#[test]
fn test_if_else_expression() {
    let tests = [
        ("if (true) {10}", Object::Integer(10)),
        ("if (false) {10}", Object::Null),
        ("if (1) {10}", Object::Integer(10)),
        ("if (1 < 2) {10}", Object::Integer(10)),
        ("if (1 > 2) { 10 }", Object::Null),
        ("if (1 > 2) { 10 } else { 20 }", Object::Integer(20)),
        ("if (1 < 2) { 10 } else { 20 }", Object::Integer(10)),
    ];

    for (input, expected) in tests.iter() {
        let mut parser = Parser::new(Lexer::new(input));
        let ast = parser.parse_program().unwrap();

        let evaluated = ast.evaluate();
        assert_eq!(evaluated, *expected);
    }
}
