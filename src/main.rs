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

use crate::ast::Node;
use ast::IntegerLiteral;
use compiler::Compiler;

fn main() {
    let mut compiler = Compiler::new();
    let node = Node::IntegerLiteral(IntegerLiteral { value: 42 });
    match compiler.compile(node) {
        Ok(_) => {
            let bytecode = compiler.bytecode();
            println!("Instructions: {:?}", bytecode.instructions);
            println!("Constants: {:?}", bytecode.constants);
        }
        Err(err) => {
            eprintln!("Compilation error: {}", err);
        }
    }
}
