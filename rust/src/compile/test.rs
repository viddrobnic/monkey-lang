use std::rc::Rc;

use crate::{
    code::{Bytecode, Instruction},
    compile::{Compiler, Result},
    object::Object,
    parse::parse,
};

struct TestCase {
    input: &'static str,
    expected: Bytecode,
}

#[test]
fn test_integer_arithemtic() -> Result<()> {
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

        let mut compiler = Compiler::new();
        compiler.compile(&program)?;

        assert_eq!(*compiler.bytecode(), case.expected);
    }

    Ok(())
}

#[test]
fn test_boolean_expression() -> Result<()> {
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

        let mut compiler = Compiler::new();
        compiler.compile(&program)?;

        assert_eq!(*compiler.bytecode(), case.expected);
    }

    Ok(())
}

#[test]
fn test_conditionals() -> Result<()> {
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

        let mut compiler = Compiler::new();
        compiler.compile(&program)?;

        assert_eq!(*compiler.bytecode(), case.expected);
    }

    Ok(())
}

#[test]
fn test_global_let_statements() -> Result<()> {
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

        let mut compiler = Compiler::new();
        compiler.compile(&program)?;

        assert_eq!(*compiler.bytecode(), case.expected);
    }

    Ok(())
}

#[test]
fn test_string_expressions() -> Result<()> {
    let tests = [
        TestCase {
            input: "\"monkey\"",
            expected: Bytecode {
                constants: vec![Object::String(Rc::new("monkey".to_string()))],
                instructions: vec![Instruction::Constant(0), Instruction::Pop],
            },
        },
        TestCase {
            input: "\"mon\" + \"key\"",
            expected: Bytecode {
                constants: vec![
                    Object::String(Rc::new("mon".to_string())),
                    Object::String(Rc::new("key".to_string())),
                ],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::Add,
                    Instruction::Pop,
                ],
            },
        },
    ];

    for case in tests {
        let program = parse(case.input).unwrap();

        let mut compiler = Compiler::new();
        compiler.compile(&program)?;

        assert_eq!(*compiler.bytecode(), case.expected);
    }

    Ok(())
}

#[test]
fn test_array_literals() -> Result<()> {
    let tests = [
        TestCase {
            input: "[]",
            expected: Bytecode {
                constants: vec![],
                instructions: vec![Instruction::Array(0), Instruction::Pop],
            },
        },
        TestCase {
            input: "[1, 2, 3]",
            expected: Bytecode {
                constants: vec![Object::Integer(1), Object::Integer(2), Object::Integer(3)],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::Constant(2),
                    Instruction::Array(3),
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "[1 + 2, 3 - 4, 5 * 6]",
            expected: Bytecode {
                constants: vec![
                    Object::Integer(1),
                    Object::Integer(2),
                    Object::Integer(3),
                    Object::Integer(4),
                    Object::Integer(5),
                    Object::Integer(6),
                ],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::Add,
                    Instruction::Constant(2),
                    Instruction::Constant(3),
                    Instruction::Sub,
                    Instruction::Constant(4),
                    Instruction::Constant(5),
                    Instruction::Mul,
                    Instruction::Array(3),
                    Instruction::Pop,
                ],
            },
        },
    ];

    for case in tests {
        let program = parse(case.input).unwrap();

        let mut compiler = Compiler::new();
        compiler.compile(&program)?;

        assert_eq!(*compiler.bytecode(), case.expected);
    }

    Ok(())
}

#[test]
fn test_hash_literals() -> Result<()> {
    let tests = [
        TestCase {
            input: "{}",
            expected: Bytecode {
                constants: vec![],
                instructions: vec![Instruction::Hash(0), Instruction::Pop],
            },
        },
        TestCase {
            input: "{1: 2, 3: 4, 5: 6}",
            expected: Bytecode {
                constants: vec![
                    Object::Integer(1),
                    Object::Integer(2),
                    Object::Integer(3),
                    Object::Integer(4),
                    Object::Integer(5),
                    Object::Integer(6),
                ],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::Constant(2),
                    Instruction::Constant(3),
                    Instruction::Constant(4),
                    Instruction::Constant(5),
                    Instruction::Hash(6),
                    Instruction::Pop,
                ],
            },
        },
        TestCase {
            input: "{1: 2 + 3, 4: 5 * 6}",
            expected: Bytecode {
                constants: vec![
                    Object::Integer(1),
                    Object::Integer(2),
                    Object::Integer(3),
                    Object::Integer(4),
                    Object::Integer(5),
                    Object::Integer(6),
                ],
                instructions: vec![
                    Instruction::Constant(0),
                    Instruction::Constant(1),
                    Instruction::Constant(2),
                    Instruction::Add,
                    Instruction::Constant(3),
                    Instruction::Constant(4),
                    Instruction::Constant(5),
                    Instruction::Mul,
                    Instruction::Hash(4),
                    Instruction::Pop,
                ],
            },
        },
    ];

    for case in tests {
        let program = parse(case.input).unwrap();

        let mut compiler = Compiler::new();
        compiler.compile(&program)?;

        assert_eq!(*compiler.bytecode(), case.expected);
    }

    Ok(())
}
