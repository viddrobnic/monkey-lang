use std::{
    fs,
    io::{stdin, stdout},
    path::PathBuf,
    process,
};

use clap::{Parser, Subcommand, ValueEnum};
use monkey::{compile::Compiler, evaluate::Evaluator, parse, repl, vm::VirtualMachine};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Runtime {
    Eval,
    Vm,
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long, value_enum, default_value_t = Runtime::Vm)]
    runtime: Runtime,

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
        None => interactive(cli.runtime),
        Some(Commands::Run { path }) => run_file(path, cli.runtime),
    }
}

fn interactive(runtime: Runtime) {
    println!("Hello! This is the Monkey programming language!");
    println!("Feel free to type in commands");
    match runtime {
        Runtime::Eval => repl::start_eval(stdin(), stdout()),
        Runtime::Vm => repl::start_vm(stdin(), stdout()),
    }
}

fn run_file(path: PathBuf, runtime: Runtime) {
    let input = fs::read_to_string(path).unwrap_or_else(|err| {
        println!("{}", err);
        process::exit(1);
    });

    let program = parse::parse(&input).unwrap_or_else(|err| {
        println!("Failed to parse input: {}", err);
        process::exit(1);
    });

    match runtime {
        Runtime::Eval => {
            let mut evaluator = Evaluator::new();
            let res = evaluator.evaluate(&program).unwrap_or_else(|err| {
                println!("Failed to run the program: {}", err);
                process::exit(1);
            });

            println!("{}", res.inspect());
        }
        Runtime::Vm => {
            let mut compiler = Compiler::new();
            let bytecode = compiler.compile(&program).unwrap_or_else(|err| {
                println!("Failed to compile the program: {}", err);
                process::exit(1);
            });

            let mut vm = VirtualMachine::new();
            vm.run(&bytecode).unwrap_or_else(|err| {
                println!("Failed to run the program: {}", err);
                process::exit(1);
            });

            println!("{}", vm.last_popped().inspect());
        }
    }
}
