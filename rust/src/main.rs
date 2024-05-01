use std::{
    fs,
    io::{stdin, stdout},
    path::PathBuf,
    process,
};

use clap::{Parser, Subcommand};
use monkey::{evaluate::Evaluator, parse, repl};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run file and print the result
    Run { path: PathBuf },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        None => interactive(),
        Some(Commands::Run { path }) => run_file(path),
    }
}

fn interactive() {
    println!("Hello! This is the Monkey programming language!");
    println!("Feel free to type in commands");
    repl::start(stdin(), stdout());
}

fn run_file(path: PathBuf) {
    let input = fs::read_to_string(path).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1);
    });

    let program = parse::parse(&input).unwrap_or_else(|err| {
        println!("Failed to parse input: {}", err);
        process::exit(1);
    });

    let mut evaluator = Evaluator::new();
    let res = evaluator.evaluate(&program).unwrap_or_else(|err| {
        println!("Failed to run the program: {}", err);
        process::exit(1);
    });

    println!("{}", res.inspect());
}
