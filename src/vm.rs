use crate::code::{Instructions, OPADD, OPCONSTANT};
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

    pub fn pop(&mut self) -> Result<Object, String> {
        if self.sp == 0 {
            return Err("Stack underflow".to_string());
        }
        self.sp -= 1;
        self.stack[self.sp]
            .take()
            .ok_or_else(|| "Failed to pop from stack".to_string())
    }

    pub fn run(&mut self) -> Result<(), String> {
        let mut ip = 0;
        while ip < self.instructions.len() {
            let opcode = self.instructions.0[ip];
            ip += 1;

            match opcode {
                OPCONSTANT => {
                    let const_index = Self::read_uint16(&self.instructions.0[ip..]);
                    ip += 2;

                    let constant = self
                        .constants
                        .get(const_index as usize)
                        .ok_or_else(|| format!("Constant not found at index {}", const_index))?;

                    self.push(constant.clone())?;
                }
                OPADD => {
                    let right = self.pop()?;
                    let left = self.pop()?;

                    match (left, right) {
                        (Object::Integer(left_val), Object::Integer(right_val)) => {
                            self.push(Object::Integer(left_val + right_val))?;
                        }
                        _ => return Err("Unsupported types for addition".to_string()),
                    }
                }
                _ => return Err(format!("Unknown opcode: {}", opcode)),
            }
        }

        Ok(())
    }

    fn read_uint16(bytes: &[u8]) -> u16 {
        ((bytes[0] as u16) << 8) | (bytes[1] as u16)
    }

    fn push(&mut self, obj: Object) -> Result<(), String> {
        if self.sp >= STACK_SIZE {
            return Err("Stack Overflow".to_string());
        }
        if self.stack.len() <= self.sp {
            self.stack.push(Some(obj));
        } else {
            self.stack[self.sp] = Some(obj);
        }
        self.sp += 1;
        Ok(())
    }
}
