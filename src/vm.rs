use crate::code::{self, Instructions, Opcode};
use crate::compiler;
use crate::object;

pub const STACK_SIZE: usize = 2048;
pub const GLOBAL_SIZE: usize = 65536;
pub const MAX_FRAMES: usize = 1024;

/// Pre-allocate singletons for True, False, Null
pub static TRUE_OBJ: object::Boolean = object::Boolean { value: true };
pub static FALSE_OBJ: object::Boolean = object::Boolean { value: false };
pub static NULL_OBJ: object::Null = object::Null;

#[derive(Debug, Clone)]
pub struct Frame {
    pub cl: object::Closure, // The closure (function + free vars)
    pub ip: usize,           // Instruction pointer
    pub base_pointer: usize, // Where local variables start
}

impl Frame {
    pub fn new(cl: object::Closure, base_pointer: usize) -> Self {
        Frame {
            cl,
            ip: 0,
            base_pointer,
        }
    }

    pub fn instructions(&self) -> &Instructions {
        &self.cl.fn_obj.instructions
    }
}

#[derive(Debug)]
pub struct VM {
    pub constants: Vec<object::Object>,

    pub stack: Vec<object::Object>,
    pub sp: usize, // "stack pointer": next value index

    pub globals: Vec<object::Object>,

    pub frames: Vec<Frame>,
    pub frames_index: usize,
}

impl VM {
    pub fn new(bytecode: &compiler::Bytecode) -> Self {
        let main_fn = object::CompiledFunction {
            instructions: bytecode.instructions.clone(),
            num_locals: 0,
            num_parameters: 0,
        };
        let main_closure = object::Closure {
            fn_obj: main_fn,
            free: vec![],
        };
        let main_frame = Frame::new(main_closure, 0);

        let mut frames = Vec::with_capacity(MAX_FRAMES);
        frames.push(main_frame);

        VM {
            constants: bytecode.constants.clone(),
            stack: vec![object::Object::Null(NULL_OBJ); STACK_SIZE],
            sp: 0,
            globals: vec![object::Object::Null(NULL_OBJ); GLOBAL_SIZE],
            frames,
            frames_index: 1,
        }
    }

    pub fn new_with_globals_store(bytecode: &compiler::Bytecode, s: Vec<object::Object>) -> Self {
        let mut vm = VM::new(bytecode);
        vm.globals = s;
        vm
    }

    pub fn run(&mut self) -> Result<(), String> {
        while self.current_frame().ip < self.current_frame().instructions().0.len() - 1 {
            self.current_frame_mut().ip += 1;
            let ip = self.current_frame().ip;
            let ins = &self.current_frame().instructions().0;
            let op =
                Opcode::from_u8(ins[ip]).ok_or_else(|| format!("Unknown opcode {}", ins[ip]))?;

            match op {
                Opcode::OpConstant => {
                    let const_index = code::read_uint16(&ins[ip + 1..]);
                    self.current_frame_mut().ip += 2;

                    let constant = self.constants[const_index as usize].clone();
                    self.push(constant)?;
                }
                Opcode::OpPop => {
                    self.pop();
                }
                Opcode::OpAdd | Opcode::OpSub | Opcode::OpMul | Opcode::OpDiv => {
                    self.execute_binary_operation(op)?;
                }
                Opcode::OpTrue => {
                    self.push(object::Object::Boolean(TRUE_OBJ))?;
                }
                Opcode::OpFalse => {
                    self.push(object::Object::Boolean(FALSE_OBJ))?;
                }
                Opcode::OpEqual | Opcode::OpNotEqual | Opcode::OpGreaterThan => {
                    self.execute_comparison(op)?;
                }
                Opcode::OpBang => {
                    self.execute_bang_operator()?;
                }
                Opcode::OpMinus => {
                    self.execute_minus_operator()?;
                }
                Opcode::OpJumpNotTruthy => {
                    let pos = code::read_uint16(&ins[ip + 1..]) as usize;
                    self.current_frame_mut().ip += 2;

                    let condition = self.pop();
                    if !is_truthy(&condition) {
                        self.current_frame_mut().ip = pos - 1;
                    }
                }
                Opcode::OpJump => {
                    let pos = code::read_uint16(&ins[ip + 1..]) as usize;
                    self.current_frame_mut().ip = pos - 1;
                }
                Opcode::OpNull => {
                    self.push(object::Object::Null(NULL_OBJ))?;
                }
                Opcode::OpSetGlobal => {
                    let global_index = code::read_uint16(&ins[ip + 1..]) as usize;
                    self.current_frame_mut().ip += 2;
                    let popped = self.pop();
                    self.globals[global_index] = popped;
                }
                Opcode::OpGetGlobal => {
                    let global_index = code::read_uint16(&ins[ip + 1..]) as usize;
                    self.current_frame_mut().ip += 2;
                    let val = self.globals[global_index].clone();
                    self.push(val)?;
                }
                Opcode::OpArray => {
                    let num_elements = code::read_uint16(&ins[ip + 1..]) as usize;
                    self.current_frame_mut().ip += 2;

                    let array = self.build_array(self.sp - num_elements, self.sp);
                    self.sp -= num_elements;
                    self.push(array)?;
                }
                Opcode::OpHash => {
                    let num_elements = code::read_uint16(&ins[ip + 1..]) as usize;
                    self.current_frame_mut().ip += 2;

                    let hash_obj = self.build_hash(self.sp - num_elements, self.sp)?;
                    self.sp -= num_elements;
                    self.push(hash_obj)?;
                }
                Opcode::OpIndex => {
                    let index = self.pop();
                    let left = self.pop();
                    self.execute_index_expression(left, index)?;
                }
                Opcode::OpCall => {
                    let num_args = code::read_uint8(&ins[ip + 1..]) as usize;
                    self.current_frame_mut().ip += 1;
                    self.execute_call(num_args)?;
                }
                Opcode::OpReturnValue => {
                    let return_value = self.pop();
                    let frame = self.pop_frame();
                    self.sp = frame.base_pointer - 1;
                    self.push(return_value)?;
                }
                Opcode::OpReturn => {
                    let frame = self.pop_frame();
                    self.sp = frame.base_pointer - 1;
                    self.push(object::Object::Null(NULL_OBJ))?;
                }
                Opcode::OpSetLocal => {
                    let local_index = code::read_uint8(&ins[ip + 1..]) as usize;
                    self.current_frame_mut().ip += 1;

                    let frame = self.current_frame();
                    let popped = self.pop();
                    self.stack[frame.base_pointer + local_index] = popped;
                }
                Opcode::OpGetLocal => {
                    let local_index = code::read_uint8(&ins[ip + 1..]) as usize;
                    self.current_frame_mut().ip += 1;

                    let frame = self.current_frame();
                    let val = self.stack[frame.base_pointer + local_index].clone();
                    self.push(val)?;
                }
                Opcode::OpGetBuiltin => {
                    let builtin_index = code::read_uint8(&ins[ip + 1..]) as usize;
                    self.current_frame_mut().ip += 1;

                    let definition = object::BUILTINS[builtin_index].clone();
                    self.push(definition.builtin)?;
                }
                Opcode::OpClosure => {
                    let const_index = code::read_uint16(&ins[ip + 1..]) as usize;
                    let num_free = code::read_uint8(&ins[ip + 3..]) as usize;
                    self.current_frame_mut().ip += 3;

                    self.push_closure(const_index, num_free)?;
                }
                Opcode::OpGetFree => {
                    let free_index = code::read_uint8(&ins[ip + 1..]) as usize;
                    self.current_frame_mut().ip += 1;

                    let current_closure = &self.current_frame().cl;
                    let val = current_closure.free[free_index].clone();
                    self.push(val)?;
                }

                _ => {
                    return Err(format!("Unhandled opcode: {:?}", op));
                }
            }
        }

        Ok(())
    }

    pub fn stack_top(&self) -> Option<object::Object> {
        if self.sp == 0 {
            None
        } else {
            Some(self.stack[self.sp - 1].clone())
        }
    }

    pub fn last_popped_stack_elem(&self) -> object::Object {
        self.stack[self.sp].clone()
    }

    fn push(&mut self, obj: object::Object) -> Result<(), String> {
        if self.sp >= STACK_SIZE {
            return Err("stack overflow".to_string());
        }
        self.stack[self.sp] = obj;
        self.sp += 1;
        Ok(())
    }

    fn pop(&mut self) -> object::Object {
        let o = self.stack[self.sp - 1].clone();
        self.sp -= 1;
        o
    }

    fn current_frame(&self) -> &Frame {
        &self.frames[self.frames_index - 1]
    }

    fn current_frame_mut(&mut self) -> &mut Frame {
        &mut self.frames[self.frames_index - 1]
    }

    fn push_frame(&mut self, f: Frame) {
        self.frames[self.frames_index] = f;
        self.frames_index += 1;
    }

    fn pop_frame(&mut self) -> Frame {
        self.frames_index -= 1;
        self.frames[self.frames_index].clone()
    }

    fn execute_binary_operation(&mut self, op: Opcode) -> Result<(), String> {
        let right = self.pop();
        let left = self.pop();

        match (left.object_type(), right.object_type()) {
            (object::ObjectType::Integer, object::ObjectType::Integer) => {
                self.execute_binary_integer_operation(op, left, right)
            }
            (object::ObjectType::String, object::ObjectType::String) => {
                self.execute_binary_string_operation(op, left, right)
            }
            (l, r) => Err(format!(
                "unsupported types for binary operation: {:?} {:?}",
                l, r
            )),
        }
    }

    fn execute_binary_integer_operation(
        &mut self,
        op: Opcode,
        left: object::Object,
        right: object::Object,
    ) -> Result<(), String> {
        let left_val = left.as_integer().unwrap();
        let right_val = right.as_integer().unwrap();

        let result = match op {
            Opcode::OpAdd => left_val + right_val,
            Opcode::OpSub => left_val - right_val,
            Opcode::OpMul => left_val * right_val,
            Opcode::OpDiv => left_val / right_val,
            _ => return Err(format!("unknown integer operator: {:?}", op)),
        };

        self.push(object::Object::Integer(object::Integer { value: result }))
    }

    fn execute_binary_string_operation(
        &mut self,
        op: Opcode,
        left: object::Object,
        right: object::Object,
    ) -> Result<(), String> {
        if op != Opcode::OpAdd {
            return Err(format!("unknown string operator: {:?}", op));
        }

        let left_val = left.as_string().unwrap();
        let right_val = right.as_string().unwrap();
        let new_str = format!("{}{}", left_val, right_val);
        self.push(object::Object::String(object::String_ { value: new_str }))
    }

    fn execute_comparison(&mut self, op: Opcode) -> Result<(), String> {
        let right = self.pop();
        let left = self.pop();

        if left.object_type() == object::ObjectType::Integer
            && right.object_type() == object::ObjectType::Integer
        {
            return self.execute_integer_comparison(op, left, right);
        }

        match op {
            Opcode::OpEqual => {
                self.push(native_bool_to_boolean_object(right == left))?;
            }
            Opcode::OpNotEqual => {
                self.push(native_bool_to_boolean_object(right != left))?;
            }
            _ => {
                return Err(format!(
                    "unknown operator: {:?} between {:?} and {:?}",
                    op,
                    left.object_type(),
                    right.object_type()
                ))
            }
        }
        Ok(())
    }

    fn execute_integer_comparison(
        &mut self,
        op: Opcode,
        left: object::Object,
        right: object::Object,
    ) -> Result<(), String> {
        let left_val = left.as_integer().unwrap();
        let right_val = right.as_integer().unwrap();

        match op {
            Opcode::OpEqual => {
                self.push(native_bool_to_boolean_object(left_val == right_val))?;
            }
            Opcode::OpNotEqual => {
                self.push(native_bool_to_boolean_object(left_val != right_val))?;
            }
            Opcode::OpGreaterThan => {
                self.push(native_bool_to_boolean_object(left_val > right_val))?;
            }
            _ => return Err(format!("unknown operator: {:?}", op)),
        }
        Ok(())
    }

    fn execute_bang_operator(&mut self) -> Result<(), String> {
        let operand = self.pop();
        match operand {
            object::Object::Boolean(b) if b.value => {
                self.push(object::Object::Boolean(FALSE_OBJ))?;
            }
            object::Object::Boolean(b) if !b.value => {
                self.push(object::Object::Boolean(TRUE_OBJ))?;
            }
            object::Object::Null(_) => {
                self.push(object::Object::Boolean(TRUE_OBJ))?;
            }
            _ => {
                self.push(object::Object::Boolean(FALSE_OBJ))?;
            }
        };
        Ok(())
    }

    fn execute_minus_operator(&mut self) -> Result<(), String> {
        let operand = self.pop();
        if let object::Object::Integer(i) = operand {
            self.push(object::Object::Integer(object::Integer { value: -i.value }))
        } else {
            Err(format!(
                "unsupported type for negation: {:?}",
                operand.object_type()
            ))
        }
    }

    fn execute_index_expression(
        &mut self,
        left: object::Object,
        index: object::Object,
    ) -> Result<(), String> {
        match (left.object_type(), index.object_type()) {
            (object::ObjectType::Array, object::ObjectType::Integer) => {
                self.execute_array_index(left, index)
            }
            (object::ObjectType::Hash, _) => self.execute_hash_index(left, index),
            (l, _) => Err(format!("index operator not supported: {:?}", l)),
        }
    }

    fn execute_array_index(
        &mut self,
        array: object::Object,
        index: object::Object,
    ) -> Result<(), String> {
        let array_obj = array.as_array().unwrap();
        let i = index.as_integer().unwrap();
        let max = array_obj.elements.len() as i64 - 1;
        if i < 0 || i > max {
            self.push(object::Object::Null(NULL_OBJ))
        } else {
            let elem = array_obj.elements[i as usize].clone();
            self.push(elem)
        }
    }

    fn execute_hash_index(
        &mut self,
        hash_obj: object::Object,
        index: object::Object,
    ) -> Result<(), String> {
        let h = hash_obj.as_hash().unwrap();
        let key = index
            .to_hash_key()
            .ok_or_else(|| format!("unusable as hash key: {:?}", index.object_type()))?;

        match h.pairs.get(&key) {
            Some(pair) => self.push(pair.value.clone()),
            None => self.push(object::Object::Null(NULL_OBJ)),
        }
    }

    fn build_array(&self, start_index: usize, end_index: usize) -> object::Object {
        let elements = self.stack[start_index..end_index].to_vec();
        object::Object::Array(object::Array { elements })
    }

    fn build_hash(&self, start_index: usize, end_index: usize) -> Result<object::Object, String> {
        let mut pairs = std::collections::HashMap::new();
        let slice = &self.stack[start_index..end_index];
        for chunk in slice.chunks_exact(2) {
            let key = &chunk[0];
            let value = &chunk[1];

            let hash_key = key
                .to_hash_key()
                .ok_or_else(|| format!("unusable as hash key: {:?}", key.object_type()))?;

            pairs.insert(
                hash_key,
                object::HashPair {
                    key: key.clone(),
                    value: value.clone(),
                },
            );
        }
        Ok(object::Object::Hash(object::Hash_ { pairs }))
    }

    fn execute_call(&mut self, num_args: usize) -> Result<(), String> {
        let callee = self.stack[self.sp - 1 - num_args].clone();
        match callee {
            object::Object::Closure(cl) => self.call_closure(cl, num_args),
            object::Object::Builtin(b) => self.call_builtin(b, num_args),
            _ => Err("calling non-function and non-built-in".to_string()),
        }
    }

    fn call_closure(&mut self, cl: object::Closure, num_args: usize) -> Result<(), String> {
        if num_args != cl.fn_obj.num_parameters {
            return Err(format!(
                "wrong number of arguments: want={}, got={}",
                cl.fn_obj.num_parameters, num_args
            ));
        }

        let base_pointer = self.sp - num_args;
        let frame = Frame::new(cl, base_pointer);
        self.push_frame(frame);

        self.sp = base_pointer + self.current_frame().cl.fn_obj.num_locals;
        Ok(())
    }

    fn call_builtin(&mut self, builtin: object::Builtin, num_args: usize) -> Result<(), String> {
        let args = &self.stack[self.sp - num_args..self.sp];
        let result = (builtin.func)(args)?;
        self.sp = self.sp - num_args - 1;

        if let Some(r) = result {
            self.push(r)
        } else {
            self.push(object::Object::Null(NULL_OBJ))
        }
    }

    fn push_closure(&mut self, const_index: usize, num_free: usize) -> Result<(), String> {
        let constant = self.constants[const_index].clone();
        let function = match constant {
            object::Object::CompiledFunction(cf) => cf,
            other => {
                return Err(format!("not a function: {:?}", other));
            }
        };

        let mut free = Vec::with_capacity(num_free);
        for i in 0..num_free {
            free.push(self.stack[self.sp - num_free + i].clone());
        }
        self.sp -= num_free;

        let closure = object::Closure {
            fn_obj: function,
            free,
        };
        self.push(object::Object::Closure(closure))
    }
}

fn native_bool_to_boolean_object(b: bool) -> object::Object {
    if b {
        object::Object::Boolean(TRUE_OBJ)
    } else {
        object::Object::Boolean(FALSE_OBJ)
    }
}

fn is_truthy(obj: &object::Object) -> bool {
    match obj {
        object::Object::Boolean(b) => b.value,
        object::Object::Null(_) => false,
        _ => true,
    }
}
