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
        let program = parse(case.input).unwrap();
        let bytecode = compile(&program);

        assert_eq!(bytecode, case.expected);
    }
}

#[test]
fn test_conditionals() {
    let tests = [
        TestCase {
            input: "if (true) { 10 }; 3333;",
            expected: Bytecode {
                constants: vec![Object::Integer(10), Object::Integer(3333)],
                instructions: vec![
                    Instruction::True,
                    Instruction::JumpNotTruthy(4),
                    Instruction::Constant(0),
                    Instruction::Jump(5),
                    Instruction::Null,
                    Instruction::Pop,
                    Instruction::Constant(1),
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "if (true) { 10 } else { 20 }; 3333;",
            expected: Bytecode {
                constants: vec![
                    Object::Integer(10),
                    Object::Integer(20),
                    Object::Integer(3333),
                ],
                instructions: vec![
                    Instruction::True,
                    Instruction::JumpNotTruthy(4),
                    Instruction::Constant(0),
                    Instruction::Jump(5),
                    Instruction::Constant(1),
                    Instruction::Pop,
                    Instruction::Constant(2),
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
fn test_global_let_statements() {
    let tests = [
        TestCase {
            input: "let one = 1; let two = 2;",
            expected: Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::SetGlobal(0),
                    Instruction::Constant(1),
                    Instruction::SetGlobal(1),
                ],
            },
        },
        TestCase {
            input: "let one = 1; one",
            expected: Bytecode {
                constants: vec![Object::Integer(1)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::SetGlobal(0),
                    Instruction::GetGlobal(0),
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "let one = 1; let two = 2; two;",
            expected: Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::SetGlobal(0),
                    Instruction::Constant(1),
                    Instruction::SetGlobal(1),
                    Instruction::GetGlobal(1),
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
