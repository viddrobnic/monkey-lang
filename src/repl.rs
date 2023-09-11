use std::io::{self, BufRead};

use crate::{lexer::Lexer, parser::Parser};

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
    loop {
        write!(output, "{}", PROMPT).unwrap();
        output.flush().unwrap();

        let mut line = String::new();
        if reader.read_line(&mut line).is_err() {
            return;
        }

        let lexer = Lexer::new(&line);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();
        match program {
            Ok(program) => {
                for stmt in program.statements {
                    writeln!(output, "{}", stmt.debug_str()).unwrap();
                }
            }
            Err(err) => {
                writeln!(output, "{}", MONKEY_FACE).unwrap();
                writeln!(output, "Woops! We ran into some monkey business here!").unwrap();
                writeln!(output, "{}", err).unwrap();
            }
        }
    }
}
