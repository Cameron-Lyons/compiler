use std::io::{BufRead, Write};

use crate::compiler;
mod lexer;
mod parser;
use crate::vm;

const PROMPT: &str = ">> ";

pub fn start<R: BufRead, W: Write>(input: R, mut output: W) {
    let mut scanner = input.lines();

    while let Some(Ok(line)) = scanner.next() {
        write!(output, "{}", PROMPT).unwrap();

        let l = lexer::Lexer::new(&line);
        let mut p = parser::Parser::new(l);

        let program = p.parse_program();
        if !p.errors().is_empty() {
            print_parser_errors(&mut output, &p.errors());
            continue;
        }

        let mut comp = compiler::Compiler::new();
        if let Err(err) = comp.compile(&program) {
            writeln!(output, "Woops! Compilation failed:\n{}", err).unwrap();
            continue;
        }

        let mut machine = vm::VM::new(comp.bytecode());
        if let Err(err) = machine.run() {
            writeln!(output, "Woops! Executing bytecode failed:\n{}", err).unwrap();
            continue;
        }

        if let Some(stack_top) = machine.stack_top() {
            writeln!(output, "{}", stack_top.inspect()).unwrap();
        } else {
            writeln!(output, "Stack is empty").unwrap();
        }
    }
}

fn print_parser_errors<W: Write>(output: &mut W, errors: &[String]) {
    for error in errors {
        writeln!(output, "Parser error: {}", error).unwrap();
    }
}
