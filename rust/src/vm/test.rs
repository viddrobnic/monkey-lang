use crate::{compile::compile, object::Object, parse::parse};

use super::{Result, VirtualMachine};

fn run_test_case(input: &str, expected: Object) -> Result<()> {
    let program = parse(input).unwrap();
    let bytecode = compile(&program);

    let mut vm = VirtualMachine::new(&bytecode);
    vm.run()?;

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
