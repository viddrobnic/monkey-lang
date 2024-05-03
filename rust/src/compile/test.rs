use crate::{
    code::{Bytecode, Instruction},
    compile::compile,
    object::Object,
    parse::parse,
};

struct TestCase {
    input: &'static str,
    expected: Bytecode,
}

#[test]
fn test_integer_arithemtic() {
    let tests = [
        TestCase {
            input: "1 + 2",
            expected: Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::Add,
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "1 - 2",
            expected: Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::Sub,
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "1 * 2",
            expected: Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::Mul,
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "2/1",
            expected: Bytecode {
                constants: vec![Object::Integer(2), Object::Integer(1)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::Div,
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "-1",
            expected: Bytecode {
                constants: vec![Object::Integer(1)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Minus,
                    Instruction::Pop,
                ],
            },
        },
    ];

    for case in tests {
        let program = parse(case.input).unwrap();
        let bytecode = compile(&program);

        assert_eq!(bytecode, case.expected);
    }
}

#[test]
fn test_boolean_expression() {
    let tests = [
        TestCase {
            input: "true",
            expected: Bytecode {
                constants: vec![],
                instructions: vec![Instruction::True, Instruction::Pop],
            },
        },
        TestCase {
            input: "false",
            expected: Bytecode {
                constants: vec![],
                instructions: vec![Instruction::False, Instruction::Pop],
            },
        },
        TestCase {
            input: "1 > 2",
            expected: Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::GreaterThan,
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "1 < 2",
            expected: Bytecode {
                constants: vec![Object::Integer(2), Object::Integer(1)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::GreaterThan,
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "1 == 2",
            expected: Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::Equal,
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "1 != 2",
            expected: Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::NotEqual,
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "true == false",
            expected: Bytecode {
                constants: vec![],
                instructions: vec![
                    Instruction::True,
                    Instruction::False,
                    Instruction::Equal,
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "true != false",
            expected: Bytecode {
                constants: vec![],
                instructions: vec![
                    Instruction::True,
                    Instruction::False,
                    Instruction::NotEqual,
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "!true",
            expected: Bytecode {
                constants: vec![],
                instructions: vec![Instruction::True, Instruction::Bang, Instruction::Pop],
            },
        },
    ];

    for case in tests {
        let program = parse(&case.input).unwrap();
        let bytecode = compile(&program);

        assert_eq!(bytecode, case.expected);
    }
}
