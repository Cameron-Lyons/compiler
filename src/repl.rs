use std::io::{self, BufRead, Write};

use crate::compiler::{Compiler, SymbolTable};
use crate::lexer::Lexer;
use crate::object::{Object, BUILTINS};
use crate::parser::Parser;
use crate::vm::{GLOBAL_SIZE, VM};

pub const PROMPT: &str = ">> ";

pub fn start<R: BufRead, W: Write>(input: &mut R, output: &mut W) {
    let mut constants: Vec<Object> = vec![];
    let mut globals: Vec<Object> = vec![Object::Null(crate::object::Null); GLOBAL_SIZE];
    let mut symbol_table = SymbolTable::new();

    for (i, builtin_def) in BUILTINS.iter().enumerate() {
        symbol_table.define_builtin(i, &builtin_def.name);
    }

    loop {
        write!(output, "{}", PROMPT).unwrap();
        output.flush().unwrap();

        let mut line = String::new();
        match input.read_line(&mut line) {
            Ok(0) => {
                return;
            }
            Ok(_) => {
                let mut lexer = Lexer::new(&line);
                let mut parser = Parser::new(&mut lexer);
                let program = parser.parse_program();

                let errors = parser.errors();
                if !errors.is_empty() {
                    print_parse_errors(output, errors);
                    continue;
                }

                let mut compiler = Compiler::new_with_state(symbol_table, constants);
                if let Err(e) = compiler.compile(&program) {
                    writeln!(output, "Woops! Compilation failed:\n {}", e).unwrap();
                    continue;
                }

                let bytecode = compiler.bytecode();
                constants = bytecode.constants.clone();
                symbol_table = compiler.symbol_table();

                let mut vm = VM::new_with_globals_store(bytecode, globals.clone());
                if let Err(e) = vm.run() {
                    writeln!(output, "Woops! Executing bytecode failed:\n {}", e).unwrap();
                    continue;
                }

                let last_popped = vm.last_popped_stack_elem();
                write!(output, "{}\n", last_popped.inspect()).unwrap();

                globals = vm.globals;
            }
            Err(_) => {
                return;
            }
        }
    }
}

fn print_parse_errors<W: Write>(output: &mut W, errors: &[String]) {
    for msg in errors {
        writeln!(output, "\t{}", msg).unwrap();
    }
}
