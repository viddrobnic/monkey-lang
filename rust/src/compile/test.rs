use std::rc::Rc;

use crate::{
    code::{Bytecode, Instruction},
    compile::{Compiler, Result},
    object::{builtin::BuiltinFunction, CompiledFunction, Object},
    parse::parse,
};

struct TestCase {
    input: &'static str,
    expected_constants: Vec<Object>,
    expected_instructions: Vec<Instruction>,
}

fn run_test_case(case: TestCase) -> Result<()> {
    let program = parse(case.input).unwrap();

    let mut compiler = Compiler::new();
    let bytecode = compiler.compile(&program)?;

    let expected_bytecode = Bytecode {
        instructions: Rc::new(case.expected_instructions),
        constants: &case.expected_constants,
    };
    assert_eq!(bytecode, expected_bytecode);

    Ok(())
}

#[test]
fn test_integer_arithemtic() -> Result<()> {
    let tests = [
        TestCase {
            input: "1 + 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Add,
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "1 - 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Sub,
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "1 * 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Mul,
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "2/1",
            expected_constants: vec![Object::Integer(2), Object::Integer(1)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Div,
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "-1",
            expected_constants: vec![Object::Integer(1)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Minus,
                Instruction::Pop,
            ],
        },
    ];

    for case in tests {
        run_test_case(case)?;
    }

    Ok(())
}

#[test]
fn test_boolean_expression() -> Result<()> {
    let tests = [
        TestCase {
            input: "true",
            expected_constants: vec![],
            expected_instructions: vec![Instruction::True, Instruction::Pop],
        },
        TestCase {
            input: "false",
            expected_constants: vec![],
            expected_instructions: vec![Instruction::False, Instruction::Pop],
        },
        TestCase {
            input: "1 > 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::GreaterThan,
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "1 < 2",
            expected_constants: vec![Object::Integer(2), Object::Integer(1)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::GreaterThan,
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "1 == 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Equal,
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "1 != 2",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::NotEqual,
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "true == false",
            expected_constants: vec![],
            expected_instructions: vec![
                Instruction::True,
                Instruction::False,
                Instruction::Equal,
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "true != false",
            expected_constants: vec![],
            expected_instructions: vec![
                Instruction::True,
                Instruction::False,
                Instruction::NotEqual,
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "!true",
            expected_constants: vec![],
            expected_instructions: vec![Instruction::True, Instruction::Bang, Instruction::Pop],
        },
    ];

    for case in tests {
        run_test_case(case)?;
    }

    Ok(())
}

#[test]
fn test_conditionals() -> Result<()> {
    let tests = [
        TestCase {
            input: "if (true) { 10 }; 3333;",
            expected_constants: vec![Object::Integer(10), Object::Integer(3333)],
            expected_instructions: vec![
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
        TestCase {
            input: "if (true) { 10 } else { 20 }; 3333;",
            expected_constants: vec![
                Object::Integer(10),
                Object::Integer(20),
                Object::Integer(3333),
            ],
            expected_instructions: vec![
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
        TestCase {
            input: "if (true) { let a = 10; }",
            expected_constants: vec![Object::Integer(10)],
            expected_instructions: vec![
                Instruction::True,
                Instruction::JumpNotTruthy(6),
                Instruction::Constant(0),
                Instruction::SetGlobal(0),
                Instruction::Null,
                Instruction::Jump(7),
                Instruction::Null,
                Instruction::Pop,
            ],
        },
    ];

    for case in tests {
        run_test_case(case)?;
    }

    Ok(())
}

#[test]
fn test_global_let_statements() -> Result<()> {
    let tests = [
        TestCase {
            input: "let one = 1; let two = 2;",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::SetGlobal(0),
                Instruction::Constant(1),
                Instruction::SetGlobal(1),
            ],
        },
        TestCase {
            input: "let one = 1; one",
            expected_constants: vec![Object::Integer(1)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::SetGlobal(0),
                Instruction::GetGlobal(0),
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "let one = 1; let two = 2; two;",
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::SetGlobal(0),
                Instruction::Constant(1),
                Instruction::SetGlobal(1),
                Instruction::GetGlobal(1),
                Instruction::Pop,
            ],
        },
    ];

    for case in tests {
        run_test_case(case)?;
    }

    Ok(())
}

#[test]
fn test_string_expressions() -> Result<()> {
    let tests = [
        TestCase {
            input: "\"monkey\"",
            expected_constants: vec![Object::String(Rc::new("monkey".to_string()))],
            expected_instructions: vec![Instruction::Constant(0), Instruction::Pop],
        },
        TestCase {
            input: "\"mon\" + \"key\"",
            expected_constants: vec![
                Object::String(Rc::new("mon".to_string())),
                Object::String(Rc::new("key".to_string())),
            ],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Add,
                Instruction::Pop,
            ],
        },
    ];

    for case in tests {
        run_test_case(case)?;
    }

    Ok(())
}

#[test]
fn test_array_literals() -> Result<()> {
    let tests = [
        TestCase {
            input: "[]",
            expected_constants: vec![],
            expected_instructions: vec![Instruction::Array(0), Instruction::Pop],
        },
        TestCase {
            input: "[1, 2, 3]",
            expected_constants: vec![Object::Integer(1), Object::Integer(2), Object::Integer(3)],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Constant(2),
                Instruction::Array(3),
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "[1 + 2, 3 - 4, 5 * 6]",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(4),
                Object::Integer(5),
                Object::Integer(6),
            ],
            expected_instructions: vec![
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
    ];

    for case in tests {
        run_test_case(case)?;
    }

    Ok(())
}

#[test]
fn test_hash_literals() -> Result<()> {
    let tests = [
        TestCase {
            input: "{}",
            expected_constants: vec![],
            expected_instructions: vec![Instruction::Hash(0), Instruction::Pop],
        },
        TestCase {
            input: "{1: 2, 3: 4, 5: 6}",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(4),
                Object::Integer(5),
                Object::Integer(6),
            ],
            expected_instructions: vec![
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
        TestCase {
            input: "{1: 2 + 3, 4: 5 * 6}",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(4),
                Object::Integer(5),
                Object::Integer(6),
            ],
            expected_instructions: vec![
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
    ];

    for case in tests {
        run_test_case(case)?;
    }

    Ok(())
}

#[test]
fn test_index_expressions() -> Result<()> {
    let tests = [
        TestCase {
            input: "[1, 2, 3][1 + 1]",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(3),
                Object::Integer(1),
                Object::Integer(1),
            ],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Constant(2),
                Instruction::Array(3),
                Instruction::Constant(3),
                Instruction::Constant(4),
                Instruction::Add,
                Instruction::Index,
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "{1: 2}[2-1]",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::Integer(2),
                Object::Integer(1),
            ],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::Constant(1),
                Instruction::Hash(2),
                Instruction::Constant(2),
                Instruction::Constant(3),
                Instruction::Sub,
                Instruction::Index,
                Instruction::Pop,
            ],
        },
    ];

    for case in tests {
        run_test_case(case)?;
    }

    Ok(())
}

#[test]
fn test_functions() -> Result<()> {
    let tests = [
        TestCase {
            input: "fn() { return 5 + 10 }",
            expected_constants: vec![
                Object::Integer(5),
                Object::Integer(10),
                Object::CompiledFunction(CompiledFunction {
                    instructions: Rc::new(vec![
                        Instruction::Constant(0),
                        Instruction::Constant(1),
                        Instruction::Add,
                        Instruction::ReturnValue,
                    ]),
                    num_locals: 0,
                    num_arguments: 0,
                }),
            ],
            expected_instructions: vec![
                Instruction::Closure {
                    constant_index: 2,
                    free_variables: 0,
                },
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "fn() { 1; 2 }",
            expected_constants: vec![
                Object::Integer(1),
                Object::Integer(2),
                Object::CompiledFunction(CompiledFunction {
                    instructions: Rc::new(vec![
                        Instruction::Constant(0),
                        Instruction::Pop,
                        Instruction::Constant(1),
                        Instruction::ReturnValue,
                    ]),
                    num_locals: 0,
                    num_arguments: 0,
                }),
            ],
            expected_instructions: vec![
                Instruction::Closure {
                    constant_index: 2,
                    free_variables: 0,
                },
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "fn() { }",
            expected_constants: vec![Object::CompiledFunction(CompiledFunction {
                instructions: Rc::new(vec![Instruction::Null, Instruction::ReturnValue]),
                num_locals: 0,
                num_arguments: 0,
            })],
            expected_instructions: vec![
                Instruction::Closure {
                    constant_index: 0,
                    free_variables: 0,
                },
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "fn() { let a = 42; }",
            expected_constants: vec![
                Object::Integer(42),
                Object::CompiledFunction(CompiledFunction {
                    instructions: Rc::new(vec![
                        Instruction::Constant(0),
                        Instruction::SetLocal(0),
                        Instruction::Null,
                        Instruction::ReturnValue,
                    ]),
                    num_locals: 1,
                    num_arguments: 0,
                }),
            ],
            expected_instructions: vec![
                Instruction::Closure {
                    constant_index: 1,
                    free_variables: 0,
                },
                Instruction::Pop,
            ],
        },
    ];

    for case in tests {
        run_test_case(case)?;
    }

    Ok(())
}

#[test]
fn test_function_calls() -> Result<()> {
    let tests = [
        TestCase {
            input: "fn() { 24 }();",
            expected_constants: vec![
                Object::Integer(24),
                Object::CompiledFunction(CompiledFunction {
                    instructions: Rc::new(vec![Instruction::Constant(0), Instruction::ReturnValue]),
                    num_locals: 0,
                    num_arguments: 0,
                }),
            ],
            expected_instructions: vec![
                Instruction::Closure {
                    constant_index: 1,
                    free_variables: 0,
                },
                Instruction::Call(0),
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "let noArg = fn() { 24 }; noArg()",
            expected_constants: vec![
                Object::Integer(24),
                Object::CompiledFunction(CompiledFunction {
                    instructions: Rc::new(vec![Instruction::Constant(0), Instruction::ReturnValue]),
                    num_locals: 0,
                    num_arguments: 0,
                }),
            ],
            expected_instructions: vec![
                Instruction::Closure {
                    constant_index: 1,
                    free_variables: 0,
                },
                Instruction::SetGlobal(0),
                Instruction::GetGlobal(0),
                Instruction::Call(0),
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "let oneArg = fn(a) { a }; oneArg(24)",
            expected_constants: vec![
                Object::CompiledFunction(CompiledFunction {
                    instructions: Rc::new(vec![Instruction::GetLocal(0), Instruction::ReturnValue]),
                    num_locals: 1,
                    num_arguments: 1,
                }),
                Object::Integer(24),
            ],
            expected_instructions: vec![
                Instruction::Closure {
                    constant_index: 0,
                    free_variables: 0,
                },
                Instruction::SetGlobal(0),
                Instruction::GetGlobal(0),
                Instruction::Constant(1),
                Instruction::Call(1),
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "let manyArg = fn(a, b, c) { a; b; c }; manyArg(24, 25, 26)",
            expected_constants: vec![
                Object::CompiledFunction(CompiledFunction {
                    instructions: Rc::new(vec![
                        Instruction::GetLocal(0),
                        Instruction::Pop,
                        Instruction::GetLocal(1),
                        Instruction::Pop,
                        Instruction::GetLocal(2),
                        Instruction::ReturnValue,
                    ]),
                    num_locals: 3,
                    num_arguments: 3,
                }),
                Object::Integer(24),
                Object::Integer(25),
                Object::Integer(26),
            ],
            expected_instructions: vec![
                Instruction::Closure {
                    constant_index: 0,
                    free_variables: 0,
                },
                Instruction::SetGlobal(0),
                Instruction::GetGlobal(0),
                Instruction::Constant(1),
                Instruction::Constant(2),
                Instruction::Constant(3),
                Instruction::Call(3),
                Instruction::Pop,
            ],
        },
    ];

    for case in tests {
        run_test_case(case)?;
    }

    Ok(())
}

#[test]
fn test_local_statement_scopes() -> Result<()> {
    let tests = [
        TestCase {
            input: "let num = 55; fn() {num}",
            expected_constants: vec![
                Object::Integer(55),
                Object::CompiledFunction(CompiledFunction {
                    instructions: Rc::new(vec![
                        Instruction::GetGlobal(0),
                        Instruction::ReturnValue,
                    ]),
                    num_locals: 0,
                    num_arguments: 0,
                }),
            ],
            expected_instructions: vec![
                Instruction::Constant(0),
                Instruction::SetGlobal(0),
                Instruction::Closure {
                    constant_index: 1,
                    free_variables: 0,
                },
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "fn() {let num = 55; num}",
            expected_constants: vec![
                Object::Integer(55),
                Object::CompiledFunction(CompiledFunction {
                    instructions: Rc::new(vec![
                        Instruction::Constant(0),
                        Instruction::SetLocal(0),
                        Instruction::GetLocal(0),
                        Instruction::ReturnValue,
                    ]),
                    num_locals: 1,
                    num_arguments: 0,
                }),
            ],
            expected_instructions: vec![
                Instruction::Closure {
                    constant_index: 1,
                    free_variables: 0,
                },
                Instruction::Pop,
            ],
        },
        TestCase {
            input: r#"
                fn () {
                    let a = 55;
                    let b = 77
                    a + b
                }"#,
            expected_constants: vec![
                Object::Integer(55),
                Object::Integer(77),
                Object::CompiledFunction(CompiledFunction {
                    instructions: Rc::new(vec![
                        Instruction::Constant(0),
                        Instruction::SetLocal(0),
                        Instruction::Constant(1),
                        Instruction::SetLocal(1),
                        Instruction::GetLocal(0),
                        Instruction::GetLocal(1),
                        Instruction::Add,
                        Instruction::ReturnValue,
                    ]),
                    num_locals: 2,
                    num_arguments: 0,
                }),
            ],
            expected_instructions: vec![
                Instruction::Closure {
                    constant_index: 2,
                    free_variables: 0,
                },
                Instruction::Pop,
            ],
        },
    ];

    for case in tests {
        run_test_case(case)?;
    }

    Ok(())
}

#[test]
fn test_builtin() -> Result<()> {
    let tests = [
        TestCase {
            input: "len([]); push([], 1);",
            expected_constants: vec![Object::Integer(1)],
            expected_instructions: vec![
                Instruction::GetBuiltin(BuiltinFunction::Len),
                Instruction::Array(0),
                Instruction::Call(1),
                Instruction::Pop,
                Instruction::GetBuiltin(BuiltinFunction::Push),
                Instruction::Array(0),
                Instruction::Constant(0),
                Instruction::Call(2),
                Instruction::Pop,
            ],
        },
        TestCase {
            input: "fn() { len([]) }",
            expected_constants: vec![Object::CompiledFunction(CompiledFunction {
                instructions: Rc::new(vec![
                    Instruction::GetBuiltin(BuiltinFunction::Len),
                    Instruction::Array(0),
                    Instruction::Call(1),
                    Instruction::ReturnValue,
                ]),
                num_locals: 0,
                num_arguments: 0,
            })],
            expected_instructions: vec![
                Instruction::Closure {
                    constant_index: 0,
                    free_variables: 0,
                },
                Instruction::Pop,
            ],
        },
    ];

    for case in tests {
        run_test_case(case)?;
    }

    Ok(())
}
