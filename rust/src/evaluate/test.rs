use std::rc::Rc;

use crate::{
    evaluate::{Error, Evaluator, HashKey, Object, Result},
    parse,
};

#[test]
fn test_eval_integer() -> Result<()> {
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
        let program = parse::parse(input).unwrap();

        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(&program)?;
        assert_eq!(result, Object::Integer(*expected));
    }

    Ok(())
}

#[test]
fn test_eval_string() -> Result<()> {
    let input = r#""Hello World!"#;

    let program = parse::parse(input).unwrap();
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&program)?;
    assert_eq!(
        result,
        Object::String(Rc::new(String::from("Hello World!")))
    );

    Ok(())
}

#[test]
fn test_string_concatenation() -> Result<()> {
    let input = r#""Hello" + " " + "World!"#;

    let program = parse::parse(input).unwrap();
    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&program)?;
    assert_eq!(
        result,
        Object::String(Rc::new(String::from("Hello World!")))
    );

    Ok(())
}

#[test]
fn test_eval_bool() -> Result<()> {
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
        let program = parse::parse(input).unwrap();

        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(&program)?;
        assert_eq!(result, Object::Boolean(*expected));
    }

    Ok(())
}

#[test]
fn test_bang_operator() -> Result<()> {
    let tests = [
        ("!true", false),
        ("!false", true),
        ("!5", false),
        ("!!true", true),
        ("!!false", false),
        ("!!5", true),
    ];

    for (input, expected) in tests.iter() {
        let program = parse::parse(input).unwrap();

        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(&program)?;
        assert_eq!(result, Object::Boolean(*expected));
    }

    Ok(())
}

#[test]
fn test_let_statements() {
    let tests = [
        ("let a = 5; a;", Object::Integer(5)),
        ("let a = 5 * 5; a", Object::Integer(25)),
        ("let a = 5; let b = a; b;", Object::Integer(5)),
        (
            "let a = 5; let b = a; let c = a + b + 5; c",
            Object::Integer(15),
        ),
    ];

    for (input, expected) in tests {
        let program = parse::parse(input).unwrap();

        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(&program).unwrap();
        assert_eq!(result, expected);
    }
}

#[test]
fn test_if_else_expression() -> Result<()> {
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
        let program = parse::parse(input).unwrap();

        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(&program)?;
        assert_eq!(result, *expected);
    }

    Ok(())
}

#[test]
fn test_return_statement() -> Result<()> {
    let tests = [
        ("return 10;", Object::Integer(10)),
        ("return 10; 9;", Object::Integer(10)),
        ("return 2 * 5; 9;", Object::Integer(10)),
        ("9; return 2 * 5; 9;", Object::Integer(10)),
        (
            "if (10 > 1) { if (10 > 1) { return 10; } return 1; }",
            Object::Integer(10),
        ),
    ];

    for (input, expected) in tests {
        let program = parse::parse(input).unwrap();

        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(&program)?;
        assert_eq!(result, expected);
    }

    Ok(())
}

#[test]
fn test_error_handling() {
    let tests = [
        (
            "5 + true;",
            Error::TypeMismatch("INTEGER + BOOLEAN".to_string()),
        ),
        (
            "5 + true; 5;",
            Error::TypeMismatch("INTEGER + BOOLEAN".to_string()),
        ),
        ("-true", Error::UnknownOperator("-BOOLEAN".to_string())),
        (
            "true + false;",
            Error::UnknownOperator("BOOLEAN + BOOLEAN".to_string()),
        ),
        (
            "5; true + false; 5",
            Error::UnknownOperator("BOOLEAN + BOOLEAN".to_string()),
        ),
        (
            "if (10 > 1) { true + false; }",
            Error::UnknownOperator("BOOLEAN + BOOLEAN".to_string()),
        ),
        (
            "if (10 > 1) { if (10 > 1) { return true + false; } return 1; }",
            Error::UnknownOperator("BOOLEAN + BOOLEAN".to_string()),
        ),
        ("foobar", Error::UnknownIdentifier("foobar".to_string())),
        (
            "\"Hello\" - \"World\"",
            Error::UnknownOperator(String::from("STRING - STRING")),
        ),
        (
            r#"{"name": "Monkey"}[fn(x) { x }];"#,
            Error::NotHashable("FUNCTION".to_string()),
        ),
    ];

    for (input, expected) in tests {
        let program = parse::parse(input).unwrap();

        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(&program);

        assert_eq!(result, Err(expected));
    }
}

#[test]
fn test_functin_object() {
    let input = "fn(x) { x + 2; };";

    let program = parse::parse(input).unwrap();

    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&program).unwrap();

    let Object::Function(fun) = result else {
        panic!("expected function object, got: {:?}", result);
    };

    assert_eq!(fun.parameters.len(), 1);
    assert_eq!(fun.parameters[0], "x");

    assert_eq!(fun.body.debug_str(), "(x + 2);");
}

#[test]
fn test_function_application() {
    let tests = [
        ("let identity = fn(x) {x}; identity(5)", Object::Integer(5)),
        (
            "let identity = fn(x) {return x;}; identity(10);",
            Object::Integer(10),
        ),
        (
            "let double = fn(x) {x * 2;}; double(5);",
            Object::Integer(10),
        ),
        (
            "let add = fn(x, y) {x + y;}; add(5, 5)",
            Object::Integer(10),
        ),
        (
            "let add = fn(x, y) {x + y;}; add(5 + 5, add(5, 5));",
            Object::Integer(20),
        ),
        ("fn(x) {x;}(5)", Object::Integer(5)),
    ];

    for (input, expected) in tests {
        let program = parse::parse(input).unwrap();

        let mut evaluator = Evaluator::new();
        let result = evaluator.evaluate(&program).unwrap();
        assert_eq!(result, expected);
    }
}

#[test]
fn test_closures() {
    let input = r#"
        let newAdder = fn(x) {
            fn(y) {x + y};
        };

        let addTwo = newAdder(2);
        addTwo(2)
    "#;

    let program = parse::parse(input).unwrap();

    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&program).unwrap();

    assert_eq!(result, Object::Integer(4));
}

#[test]
fn test_recursion() {
    let input = r#"
        let fibonacci = fn(x) {
            if (x < 3) {
                return 1;
            } else {
                return fibonacci(x - 1) + fibonacci(x - 2);
            }
        };

        fibonacci(5);
        "#;

    let program = parse::parse(input).unwrap();

    let mut evaluator = Evaluator::new();
    let result = evaluator.evaluate(&program).unwrap();

    assert_eq!(result, Object::Integer(5));
}

#[test]
fn test_array_literal() -> Result<()> {
    let input = "[1, 2 * 2, 3 + 3]";

    let program = parse::parse(input).unwrap();

    let mut evaluator = Evaluator::new();
    let res = evaluator.evaluate(&program)?;

    assert_eq!(
        res,
        Object::Array(Rc::new(vec![
            Object::Integer(1),
            Object::Integer(4),
            Object::Integer(6)
        ]))
    );

    Ok(())
}

#[test]
fn test_hash_literal() -> Result<()> {
    let input = r#"
        let two = "two";
        {
            "one": 10 - 9,
            two: 1 + 1,
            "thr" + "ee": 6 / 2,
            4: 4,
            true: 5,
            false: 6
        }"#;

    let program = parse::parse(input).unwrap();

    let mut evaluator = Evaluator::new();
    let res = evaluator.evaluate(&program)?;

    let Object::HashMap(map) = res else {
        panic!("Expected hash map, got: {:?}", res);
    };

    let expected = [
        (
            HashKey::String(Rc::new("one".to_string())),
            Object::Integer(1),
        ),
        (
            HashKey::String(Rc::new("two".to_string())),
            Object::Integer(2),
        ),
        (
            HashKey::String(Rc::new("three".to_string())),
            Object::Integer(3),
        ),
        (HashKey::Integer(4), Object::Integer(4)),
        (HashKey::Boolean(true), Object::Integer(5)),
        (HashKey::Boolean(false), Object::Integer(6)),
    ];

    for (key, value) in expected.iter() {
        assert_eq!(map.get(key), Some(value));
    }

    Ok(())
}

#[test]
fn test_array_index() -> Result<()> {
    let tests = [
        ("[1, 2, 3][0]", Object::Integer(1)),
        ("[1, 2, 3][1]", Object::Integer(2)),
        ("[1, 2, 3][2]", Object::Integer(3)),
        ("let i = 0; [1][i];", Object::Integer(1)),
        ("[1, 2, 3][1 + 1];", Object::Integer(3)),
        ("let myArray = [1, 2, 3]; myArray[2];", Object::Integer(3)),
        (
            "let myArray = [1, 2, 3]; myArray[0] + myArray[1] + myArray[2];",
            Object::Integer(6),
        ),
        (
            "let myArray = [1, 2, 3]; let i = myArray[0]; myArray[i]",
            Object::Integer(2),
        ),
        ("[1,2,3][3]", Object::Null),
        ("[1,2,3][-1]", Object::Null),
    ];

    for (input, expected) in tests {
        let program = parse::parse(input).unwrap();

        let mut evaluator = Evaluator::new();
        let res = evaluator.evaluate(&program)?;

        assert_eq!(res, expected);
    }

    Ok(())
}

#[test]
fn test_hash_index() -> Result<()> {
    let tests = [
        (r#"{"foo": 5}["foo"]"#, Object::Integer(5)),
        (r#"{"foo": 5}["bar"]"#, Object::Null),
        (r#"let key = "foo"; {"foo": 5}[key]"#, Object::Integer(5)),
        (r#"{}["foo"]"#, Object::Null),
        (r#"{5:5}[5]"#, Object::Integer(5)),
        (r#"{true : 5}[true]"#, Object::Integer(5)),
        (r#"{false: 5}[false]"#, Object::Integer(5)),
    ];

    for (input, expected) in tests {
        let program = parse::parse(input).unwrap();

        let mut evaluator = Evaluator::new();
        let res = evaluator.evaluate(&program)?;

        assert_eq!(res, expected);
    }

    Ok(())
}

#[test]
fn test_builtin_functions() {
    let tests = [
        ("len(\"\")", Ok(Object::Integer(0))),
        ("len(\"four\")", Ok(Object::Integer(4))),
        ("len(\"hello world\")", Ok(Object::Integer(11))),
        ("len(1)", Err(Error::TypeMismatch("INTEGER".to_string()))),
        (
            "len(\"one\", \"two\")",
            Err(Error::WrongNumberOfArguments {
                expected: 1,
                got: 2,
            }),
        ),
        (
            r#"
            let map = fn(arr, f) {
                let iter = fn(arr, accumulated) {
                    if (len(arr) == 0) {
                        accumulated
                    } else {
                        iter(rest(arr), push(accumulated, f(first(arr))));
                    }
                };
                iter(arr, []);
            };

            let a = [1, 2, 3, 4];
            let double = fn(x) { x * 2 };
            map(a, double);
        "#,
            Ok(Object::Array(Rc::new(vec![
                Object::Integer(2),
                Object::Integer(4),
                Object::Integer(6),
                Object::Integer(8),
            ]))),
        ),
        (
            r#"
            let reduce = fn(arr, initial, f) {
                let iter = fn(arr, result) {
                    if (len(arr) == 0) {
                        result
                    } else {
                        iter(rest(arr), f(result, first(arr)));
                    }
                };
                iter(arr, initial);
            };

            let sum = fn(arr) {
                reduce(arr, 0, fn(initial, el) { initial + el });
            };

            sum([1, 2, 3, 4, 5]);
            "#,
            Ok(Object::Integer(15)),
        ),
    ];

    for t in tests {
        let program = parse::parse(t.0).unwrap();
        let mut evaluator = Evaluator::new();
        let res = evaluator.evaluate(&program);
        assert_eq!(res, t.1);
    }
}
