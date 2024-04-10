use std::{
    fmt::Display,
    io::{self, BufRead},
};

use crate::{
    evaluate::{Evaluator, Object},
    parse,
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

pub fn start(input: impl io::Read, mut output: impl io::Write) {
    let mut reader = io::BufReader::new(input);
    let mut evaluator = Evaluator::new();

    loop {
        write!(output, "{}", PROMPT).unwrap();
        output.flush().unwrap();

        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            return;
        }

        match parse::parse(&line) {
            Ok(program) => match evaluator.evaluate(&program) {
                Ok(result) if result != Object::Null => {
                    writeln!(output, "{}", result.inspect()).unwrap()
                }
                Err(err) => writeln!(output, "{}", err).unwrap(),
                _ => (),
            },
            Err(err) => write_err(&mut output, err),
        }
    }
}

fn write_err(output: &mut impl io::Write, err: impl Display) {
    writeln!(output, "{}", MONKEY_FACE).unwrap();
    writeln!(output, "Woops! We ran into some monkey business here!").unwrap();
    writeln!(output, "{}", err).unwrap();
}
