use std::{collections::HashMap, rc::Rc};

use crate::{
    compile::Compiler,
    object::{HashKey, Object},
    parse::parse,
};

use super::{Result, VirtualMachine};

fn run_test_case(input: &str, expected: Object) -> Result<()> {
    let program = parse(input).unwrap();

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program).unwrap();

    let mut vm = VirtualMachine::new();
    vm.run(&bytecode)?;

    assert_eq!(*vm.last_popped(), expected);

    Ok(())
}

#[test]
fn test_integer_arithmetic() -> Result<()> {
    let tests = [
        ("1", Object::Integer(1)),
        ("2", Object::Integer(2)),
        ("1 + 2", Object::Integer(3)),
        ("1 - 2", Object::Integer(-1)),
        ("1 * 2", Object::Integer(2)),
        ("4 / 2", Object::Integer(2)),
        ("50 / 2 * 2 + 10 - 5", Object::Integer(55)),
        ("5 + 5 + 5 + 5 - 10", Object::Integer(10)),
        ("2 * 2 * 2 * 2 * 2", Object::Integer(32)),
        ("5 * 2 + 10", Object::Integer(20)),
        ("5 + 2 * 10", Object::Integer(25)),
        ("5 * (2 + 10)", Object::Integer(60)),
        ("-5", Object::Integer(-5)),
        ("-10", Object::Integer(-10)),
        ("-50 + 100 + -50", Object::Integer(0)),
        ("(5 + 10 * 2 + 15 / 3) * 2 + -10", Object::Integer(50)),
    ];

    for (input, expected) in tests {
        run_test_case(input, expected)?;
    }

    Ok(())
}

#[test]
fn test_boolean_expression() -> Result<()> {
    let tests = [
        ("true", Object::Boolean(true)),
        ("false", Object::Boolean(false)),
        ("1 < 2", Object::Boolean(true)),
        ("1 > 2", Object::Boolean(false)),
        ("1 < 1", Object::Boolean(false)),
        ("1 > 1", Object::Boolean(false)),
        ("1 == 1", Object::Boolean(true)),
        ("1 != 1", Object::Boolean(false)),
        ("1 == 2", Object::Boolean(false)),
        ("1 != 2", Object::Boolean(true)),
        ("true == true", Object::Boolean(true)),
        ("false == false", Object::Boolean(true)),
        ("true == false", Object::Boolean(false)),
        ("true != false", Object::Boolean(true)),
        ("false != true", Object::Boolean(true)),
        ("(1 < 2) == true", Object::Boolean(true)),
        ("(1 < 2) == false", Object::Boolean(false)),
        ("(1 > 2) == true", Object::Boolean(false)),
        ("(1 > 2) == false", Object::Boolean(true)),
        ("!true", Object::Boolean(false)),
        ("!false", Object::Boolean(true)),
        ("!5", Object::Boolean(false)),
        ("!!true", Object::Boolean(true)),
        ("!!false", Object::Boolean(false)),
        ("!!5", Object::Boolean(true)),
        ("!(if (false) { 5; })", Object::Boolean(true)),
    ];

    for (input, expected) in tests {
        run_test_case(input, expected)?;
    }

    Ok(())
}

#[test]
fn test_conditionals() -> Result<()> {
    let tests = [
        ("if (true) { 10 }", Object::Integer(10)),
        ("if (true) { 10 } else { 20 }", Object::Integer(10)),
        ("if (false) { 10 } else { 20 } ", Object::Integer(20)),
        ("if (1) { 10 }", Object::Integer(10)),
        ("if (1 < 2) { 10 }", Object::Integer(10)),
        ("if (1 < 2) { 10 } else { 20 }", Object::Integer(10)),
        ("if (1 > 2) { 10 } else { 20 }", Object::Integer(20)),
        ("if (1 > 2) { 10 }", Object::Null),
        ("if (false) { 10 }", Object::Null),
        (
            "if ((if (false) { 10 })) { 10 } else { 20 }",
            Object::Integer(20),
        ),
    ];

    for (input, expected) in tests {
        run_test_case(input, expected)?;
    }

    Ok(())
}

#[test]
fn test_global_let_statements() -> Result<()> {
    let tests = [
        ("let one = 1; one", Object::Integer(1)),
        ("let one = 1; let two = 2; one + two", Object::Integer(3)),
        (
            "let one = 1; let two = one + one; one + two",
            Object::Integer(3),
        ),
    ];

    for (input, expected) in tests {
        run_test_case(input, expected)?;
    }

    Ok(())
}

#[test]
fn test_string_expressions() -> Result<()> {
    let tests = [
        (r#""monkey""#, Object::String(Rc::new("monkey".to_string()))),
        (
            r#""mon" + "key""#,
            Object::String(Rc::new("monkey".to_string())),
        ),
        (
            r#""mon" + "key" + "banana""#,
            Object::String(Rc::new("monkeybanana".to_string())),
        ),
    ];

    for (input, expected) in tests {
        run_test_case(input, expected)?;
    }

    Ok(())
}

#[test]
fn test_array_literals() -> Result<()> {
    let tests = [
        ("[]", Object::Array(Rc::new(vec![]))),
        (
            "[1, 2, 3]",
            Object::Array(Rc::new(vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
            ])),
        ),
        (
            "[1 + 2, 3 * 4, 5 + 6]",
            Object::Array(Rc::new(vec![
                Object::Integer(3),
                Object::Integer(12),
                Object::Integer(11),
            ])),
        ),
    ];

    for (input, expected) in tests {
        run_test_case(input, expected)?;
    }

    Ok(())
}

#[test]
fn test_hash_literals() -> Result<()> {
    let tests = [
        ("{}", Object::HashMap(Rc::new(HashMap::from([])))),
        (
            "{1: 2, 2: 3}",
            Object::HashMap(Rc::new(HashMap::from([
                (HashKey::Integer(1), Object::Integer(2)),
                (HashKey::Integer(2), Object::Integer(3)),
            ]))),
        ),
        (
            "{1 + 1: 2 * 2, 3 + 3: 4 * 4}",
            Object::HashMap(Rc::new(HashMap::from([
                (HashKey::Integer(2), Object::Integer(4)),
                (HashKey::Integer(6), Object::Integer(16)),
            ]))),
        ),
    ];

    for (input, expected) in tests {
        run_test_case(input, expected)?;
    }

    Ok(())
}

#[test]
fn test_index_expressions() -> Result<()> {
    let tests = [
        ("[1, 2, 3][1]", Object::Integer(2)),
        ("[1, 2, 3][0 + 2]", Object::Integer(3)),
        ("[[1, 1, 1]][0][0]", Object::Integer(1)),
        ("[][0]", Object::Null),
        ("[1, 2, 3][99]", Object::Null),
        ("[1][-1]", Object::Null),
        ("{1: 1, 2: 2}[1]", Object::Integer(1)),
        ("{1: 1, 2: 2}[2]", Object::Integer(2)),
        ("{1: 1}[0]", Object::Null),
        ("{}[0]", Object::Null),
    ];

    for (input, expected) in tests {
        run_test_case(input, expected)?;
    }

    Ok(())
}

#[test]
fn calling_functions() -> Result<()> {
    let tests = [
        (
            "let fivePlusTen = fn() {5 + 10}; fivePlusTen()",
            Object::Integer(15),
        ),
        (
            r#"let one = fn() { 1; };
               let two = fn() { 2; };
               one() + two()"#,
            Object::Integer(3),
        ),
        (
            r#"let a = fn() { 1 };
                let b = fn() { a() + 1 };
                let c = fn() { b() + 1 };
                c();"#,
            Object::Integer(3),
        ),
        (
            "let earlyExit = fn() { return 99; 100; }; earlyExit();",
            Object::Integer(99),
        ),
        (
            "let earlyExit = fn() { return 99; return 100; }; earlyExit();",
            Object::Integer(99),
        ),
        ("let noReturn = fn() { }; noReturn();", Object::Null),
        (
            r#"
            let noReturn = fn() { };
            let noReturnTwo = fn() { noReturn(); };
            noReturn();
            noReturnTwo();"#,
            Object::Null,
        ),
        (
            r#"
            let returnsOne = fn() { 1; };
            let returnsOneReturner = fn() { returnsOne; };
            returnsOneReturner()();"#,
            Object::Integer(1),
        ),
    ];

    for (input, expected) in tests {
        run_test_case(input, expected)?;
    }

    Ok(())
}

#[test]
fn test_calling_functions_with_bindings() -> Result<()> {
    let tests = [
        (
            r#"let one = fn() { let one = 1; one };
                one();"#,
            Object::Integer(1),
        ),
        (
            r#"let oneAndTwo = fn() { let one = 1; let two = 2; one + two; };
                oneAndTwo();"#,
            Object::Integer(3),
        ),
        (
            r#"let oneAndTwo = fn() { let one = 1; let two = 2; one + two; };
                let threeAndFour = fn() { let three = 3; let four = 4; three + four; };
                oneAndTwo() + threeAndFour();"#,
            Object::Integer(10),
        ),
        (
            r#"let firstFoobar = fn() { let foobar = 50; foobar; };
                let secondFoobar = fn() { let foobar = 100; foobar; };
                firstFoobar() + secondFoobar();"#,
            Object::Integer(150),
        ),
        (
            r#"let globalSeed = 50;
                let minusOne = fn() {
                    let num = 1;
                    globalSeed - num;
                }
                let minusTwo = fn() {
                    let num = 2;
                    globalSeed - num;
                }
                minusOne() + minusTwo();"#,
            Object::Integer(97),
        ),
    ];

    for (input, expected) in tests {
        run_test_case(input, expected)?;
    }

    Ok(())
}
