// main.rs
mod ast;
mod code;
mod compiler;
mod compiler_test;
mod object;

use crate::ast::Node;
use compiler::Compiler;

fn main() {
    let mut compiler = Compiler::new();
    let node = Node::IntegerLiteral(42);
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
