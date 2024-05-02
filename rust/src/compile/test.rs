use crate::{
    code::{Bytecode, Instruction},
    object::Object,
    parse::parse,
};

use super::Compiler;

struct TestCase {
    input: &'static str,
    expected: Bytecode,
}

#[test]
fn test_integer_arithemtic() {
    let tests = [TestCase {
        input: "1 + 2",
        expected: Bytecode {
            constants: vec![Object::Integer(1), Object::Integer(2)],
            instructions: vec![Instruction::Constant(0), Instruction::Constant(1)],
        },
    }];

    for case in tests {
        let program = parse(case.input).unwrap();

        let mut compiler = Compiler::new();
        compiler.compile(&program);

        assert_eq!(*compiler.bytecode(), case.expected);
    }
}
