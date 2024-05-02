use crate::{compile::compile, object::Object, parse::parse};

use super::VirtualMachine;

#[test]
fn test_integer_arithmetic() {
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
        vm.run();

        assert_eq!(*vm.last_popped(), expected);
    }
}
