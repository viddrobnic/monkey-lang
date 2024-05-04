use std::{
    fmt::Display,
    io::{self, BufRead, BufReader},
};

use crate::{
    ast, compile::Compiler, evaluate::Evaluator, object::Object, parse::parse, vm::VirtualMachine,
};

const PROMPT: &str = ">> ";

const MONKEY_FACE: &str = r#"
            __,__
   .--.  .-"     "-.  .--.
  / .. \/  .-. .-.  \/ .. \
 | |  '|  /   Y   \  |'  | |
 | \   \  \ 0 | 0 /  /   / |
  \ '- ,\.-"""""""-./, -' /
   ''-' /_   ^ ^   _\ '-''
       |  \._   _./  |
       \   \ '~' /   /
        '._ '-=-' _.'
           '-----'
"#;

fn parse_line<R: io::Read, W: io::Write>(
    reader: &mut BufReader<R>,
    output: &mut W,
) -> Option<ast::Program> {
    write!(output, "{}", PROMPT).unwrap();
    output.flush().unwrap();

    let mut line = String::new();
    reader.read_line(&mut line).unwrap();

    match parse(&line) {
        Ok(p) => Some(p),
        Err(err) => {
            write_err(output, err);
            None
        }
    }
}

pub fn start_eval(input: impl io::Read, mut output: impl io::Write) {
    let mut reader = io::BufReader::new(input);
    let mut evaluator = Evaluator::new();

    loop {
        let Some(program) = parse_line(&mut reader, &mut output) else {
            continue;
        };

        match evaluator.evaluate(&program) {
            Ok(result) if result != Object::Null => {
                writeln!(output, "{}", result.inspect()).unwrap()
            }
            Err(err) => writeln!(output, "{}", err).unwrap(),
            _ => (),
        }
    }
}

pub fn start_vm(input: impl io::Read, mut output: impl io::Write) {
    let mut reader = io::BufReader::new(input);

    let mut compiler = Compiler::new();
    let mut vm = VirtualMachine::new();

    loop {
        let Some(program) = parse_line(&mut reader, &mut output) else {
            continue;
        };

        if let Err(err) = compiler.compile(&program) {
            writeln!(output, "Woops! Compiling the program failed: {}", err).unwrap();
            continue;
        }

        if let Err(err) = vm.run(compiler.bytecode()) {
            writeln!(output, "Woops! Executing bytecode failed: {}", err).unwrap();
            continue;
        }

        writeln!(output, "{}", vm.last_popped().inspect()).unwrap();
    }
}

fn write_err(output: &mut impl io::Write, err: impl Display) {
    writeln!(output, "{}", MONKEY_FACE).unwrap();
    writeln!(output, "Woops! We ran into some monkey business here!").unwrap();
    writeln!(output, "{}", err).unwrap();
}
