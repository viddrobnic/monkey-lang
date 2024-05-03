use crate::{compile::compile, object::Object, parse::parse};

use super::{Result, VirtualMachine};

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
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let bytecode = compile(&program);

        let mut vm = VirtualMachine::new(&bytecode);
        vm.run()?;

        assert_eq!(*vm.last_popped(), expected);
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
    ];

    for (input, expected) in tests {
        let program = parse(input).unwrap();
        let bytecode = compile(&program);

        let mut vm = VirtualMachine::new(&bytecode);
        vm.run()?;

        assert_eq!(*vm.last_popped(), expected);
    }

    Ok(())
}
