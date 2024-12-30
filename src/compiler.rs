use crate::ast::Node;
use crate::code::{make, Instructions, OPCONSTANT};
use crate::object::Object;

pub struct Compiler {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler {
            instructions: Instructions::new(Vec::new()),
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
            Node::IntegerLiteral(int_lit) => {
                let integer_obj = Object::Integer(int_lit.value);
                let constant_index = self.add_constant(integer_obj);
                self.emit(OPCONSTANT, &[constant_index as i32])?;
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

    fn add_constant(&mut self, obj: Object) -> usize {
        self.constants.push(obj);
        self.constants.len() - 1
    }

    fn emit(&mut self, op: u8, operands: &[i32]) -> Result<usize, String> {
        let instruction = make(op, operands)?;
        self.add_instruction(instruction)
    }

    fn add_instruction(&mut self, instruction: Vec<u8>) -> Result<usize, String> {
        let pos_new_instruction = self.instructions.0.len();
        self.instructions.0.extend(instruction);
        Ok(pos_new_instruction)
    }
}

pub struct Bytecode {
    pub instructions: Instructions,
    pub constants: Vec<Object>,
}
