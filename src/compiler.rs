use crate::ast::Node;
use crate::code::Instructions;
use crate::object::Object;

pub struct Compiler {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instructions: Instructions::new(),
            constants: Vec::new(),
        }
    }

    pub fn compile(&mut self, node: Node) -> Result<(), String> {
        match node {
            Node::Program(program) => {
                for stmt in program.statements {
                    self.compile(stmt)?;
                }
                Ok(())
            }
            Node::ExpressionStatement(expr_stmt) => {
                self.compile(*expr_stmt.expression)?;
                Ok(())
            }
            Node::InfixExpression(infix_expr) => {
                self.compile(*infix_expr.left)?;
                self.compile(*infix_expr.right)?;
                Ok(())
            }
            Node::IntegerLiteral(_int_lit) => {
                // TODO: emit code to load integer literal, etc.
                Ok(())
            }
        }
    }

    pub fn bytecode(&self) -> Bytecode {
        Bytecode {
            instructions: self.instructions.clone(),
            constants: self.constants.clone(),
        }
    }
}

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}
