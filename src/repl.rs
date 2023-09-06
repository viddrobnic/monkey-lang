use std::io::{self, BufRead};

use crate::{lexer::Lexer, token::Token};

const PROMPT: &str = ">> ";

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
        for token in lexer.filter(|token| token != &Token::Eof) {
            writeln!(output, "{:?}", token).unwrap();
        }
    }
}
