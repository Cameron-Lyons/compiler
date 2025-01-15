mod ast;
mod code;
mod compiler;
mod compiler_test;
mod lexer;
mod object;
mod parser;
mod repl;
mod token;
mod vm;

use std::env;
use std::io::{self, BufRead, Write};

fn main() {
    let username = env::var("USER").unwrap_or_else(|_| "User".to_string());

    println!(
        "Hello {}! This is the Monkey programming language!",
        username
    );
    println!("Feel free to type in commands");

    start_repl();
}

fn start_repl() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!(">> ");
        stdout.flush().expect("Failed to flush stdout");

        let mut buffer = String::new();
        let bytes_read = stdin
            .lock()
            .read_line(&mut buffer)
            .expect("Failed to read line from stdin");

        if bytes_read == 0 {
            break;
        }

        println!("You typed: {}", buffer.trim());
    }
}
