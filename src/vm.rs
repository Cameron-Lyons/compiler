use crate::code::Instructions;
use crate::compiler::Bytecode;
use crate::object::Object;

const STACK_SIZE: usize = 2048;

pub struct VM {
    constants: Vec<Object>,
    instructions: Instructions,
    stack: Vec<Option<Object>>,
    sp: usize,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        VM {
            instructions: bytecode.instructions,
            constants: bytecode.constants,
            stack: Vec::with_capacity(STACK_SIZE),
            sp: 0,
        }
    }

    pub fn stack_top(&self) -> Option<&Object> {
        if self.sp == 0 {
            return None;
        }
        self.stack.get(self.sp - 1).and_then(|v| v.as_ref())
    }

    pub fn run(&self) -> Result<(), String> {
        let mut ip = 0;
        while ip < self.instructions.len() {
            let opcode = self.instructions.0[ip];
            ip += 1;
            // match opcode {}
        }
        Ok(())
    }
}
