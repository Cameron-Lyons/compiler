use lexer::Lexer;
use parser::Parser;
use std::io::stdin;

pub fn main() {
    loop {
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        if input.trim_end().is_empty() {
            println!("bye");
            std::process::exit(0)
        }

        let lexer = Lexer::new(&input);
        let mut parser = Parser::new(lexer);
        match parser.parse_program() {
            Ok(program) => println!("{}", program),
            Err(errors) => {
                for error in errors {
                    eprintln!("parse error: {}", error);
                }
            }
        }
    }
}
