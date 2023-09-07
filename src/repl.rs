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

        let mut lexer = Lexer::new(&line);
        let mut token = lexer.next_token();
        while token != Token::Eof {
            writeln!(output, "{:?}", token).unwrap();
            token = lexer.next_token();
        }
    }
}
