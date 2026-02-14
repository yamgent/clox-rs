mod chunk;
mod compiler;
mod debug;
mod scanner;
mod value;
mod vm;

use std::{
    env, fs,
    io::{self, Write},
    process,
};

use crate::vm::{InterpretError, VM};

fn main() {
    let args = env::args().into_iter().collect::<Vec<_>>();

    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        run_file(args[1].clone());
    } else {
        eprintln!("Usage: clox [path]");
        process::exit(64);
    }
}

fn repl() {
    let mut vm = VM::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap_or_else(|_| {
            panic!("cannot write to stdout");
        });

        let mut buffer = String::new();

        if let Ok(total_bytes) = io::stdin().read_line(&mut buffer) {
            if total_bytes == 0 {
                // Ctrl+D will produce 0 bytes (even a blank line is one character due to \n)
                println!();
                break;
            }

            // TODO: do we to handle the result here?
            let _ = vm.interpret(buffer);
        } else {
            // EOF
            break;
        }
    }
}

fn run_file<S: AsRef<str>>(path: S) {
    let mut vm = VM::new();

    let source = match fs::read_to_string(path.as_ref()) {
        Ok(content) => content,
        Err(_) => {
            eprintln!("Could not read file {}", path.as_ref());
            process::exit(74);
        }
    };

    if let Err(error) = vm.interpret(source) {
        match error {
            InterpretError::CompileError => {
                process::exit(65);
            }
            InterpretError::RuntimeError => {
                process::exit(70);
            }
        }
    }
}
