use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

use byteorder::{BigEndian, ByteOrder};
use object::builtins::BuiltIns;

use object::{Closure, HashKey, Object};

use crate::compiler::Bytecode;
use crate::frame::Frame;
use crate::op_code::{Opcode, cast_u8_to_opcode};

const STACK_SIZE: usize = 2048;
pub const GLOBAL_SIZE: usize = 65536;
const MAX_FRAMES: usize = 1024;

#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    Null,
    Object(Rc<Object>),
}

impl Value {
    pub fn from_object(obj: Rc<Object>) -> Value {
        match &*obj {
            Object::Integer(i) => Value::Integer(*i),
            Object::Boolean(b) => Value::Boolean(*b),
            Object::Null => Value::Null,
            _ => Value::Object(obj),
        }
    }

    pub fn into_rc_object(&self) -> Rc<Object> {
        match self {
            Value::Integer(i) => Rc::new(Object::Integer(*i)),
            Value::Boolean(b) => Rc::new(Object::Boolean(*b)),
            Value::Null => Rc::new(Object::Null),
            Value::Object(o) => Rc::clone(o),
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Null => false,
            _ => true,
        }
    }

    fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "INTEGER",
            Value::Boolean(_) => "BOOLEAN",
            Value::Null => "NULL",
            Value::Object(o) => match &**o {
                Object::String(_) => "STRING",
                Object::Array(_) => "ARRAY",
                Object::Hash(_) => "HASH",
                Object::ClosureObj(_) => "CLOSURE",
                Object::Builtin(_) => "BUILTIN",
                _ => "OBJECT",
            },
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Integer(i) => write!(f, "{}", i),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Null => write!(f, "null"),
            Value::Object(o) => write!(f, "{}", o),
        }
    }
}

#[derive(Debug)]
pub enum VMError {
    StackOverflow,
    TypeError(String),
    WrongArity { expected: usize, got: usize },
    NotCallable(String),
    IndexError(String),
}

impl fmt::Display for VMError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VMError::StackOverflow => write!(f, "stack overflow"),
            VMError::TypeError(msg) => write!(f, "type error: {}", msg),
            VMError::WrongArity { expected, got } => {
                write!(
                    f,
                    "wrong number of arguments: want={}, got={}",
                    expected, got
                )
            }
            VMError::NotCallable(msg) => write!(f, "not callable: {}", msg),
            VMError::IndexError(msg) => write!(f, "index error: {}", msg),
        }
    }
}

pub struct VM {
    constants: Vec<Value>,

    stack: Vec<Value>,
    sp: usize,

    pub globals: Vec<Value>,

    frames: Vec<Frame>,
    frame_index: usize,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> VM {
        let empty_frame = Frame::new(
            Closure {
                func: Rc::from(object::CompiledFunction {
                    instructions: vec![],
                    num_locals: 0,
                    num_parameters: 0,
                }),
                free: vec![],
            },
            0,
        );

        let main_fn = Rc::from(object::CompiledFunction {
            instructions: bytecode.instructions.bytes,
            num_locals: 0,
            num_parameters: 0,
        });
        let main_closure = Closure {
            func: main_fn,
            free: vec![],
        };
        let main_frame = Frame::new(main_closure, 0);
        let mut frames = vec![empty_frame; MAX_FRAMES];
        frames[0] = main_frame;

        let constants = bytecode
            .constants
            .into_iter()
            .map(Value::from_object)
            .collect();

        VM {
            constants,
            stack: (0..STACK_SIZE).map(|_| Value::Null).collect(),
            sp: 0,
            globals: (0..GLOBAL_SIZE).map(|_| Value::Null).collect(),
            frames,
            frame_index: 1,
        }
    }

    pub fn new_with_global_store(bytecode: Bytecode, globals: Vec<Value>) -> VM {
        let mut vm = VM::new(bytecode);
        vm.globals = globals;
        vm
    }

    pub fn run(&mut self) -> Result<(), VMError> {
        let mut ins: Vec<u8>;
        while self.current_frame().ip < self.current_frame().instructions().bytes.len() as i32 - 1 {
            self.current_frame().ip += 1;
            let ip = self.current_frame().ip as usize;
            ins = self.current_frame().instructions().bytes.clone();

            let op: u8 = *ins.get(ip).unwrap();
            let opcode = cast_u8_to_opcode(op);

            match opcode {
                Opcode::OpConst => {
                    let const_index = BigEndian::read_u16(&ins[ip + 1..ip + 3]) as usize;
                    self.current_frame().ip += 2;
                    let val = self.constants[const_index].clone();
                    self.push(val)?;
                }
                Opcode::OpAdd
                | Opcode::OpSub
                | Opcode::OpMul
                | Opcode::OpDiv
                | Opcode::OpModulo => {
                    self.execute_binary_operation(opcode)?;
                }
                Opcode::OpPop => {
                    self.pop();
                }
                Opcode::OpTrue => {
                    self.push(Value::Boolean(true))?;
                }
                Opcode::OpFalse => {
                    self.push(Value::Boolean(false))?;
                }
                Opcode::OpEqual | Opcode::OpNotEqual | Opcode::OpGreaterThan => {
                    self.execute_comparison(opcode)?;
                }
                Opcode::OpMinus => {
                    self.execute_minus_operation()?;
                }
                Opcode::OpBang => {
                    self.execute_bang_operation()?;
                }
                Opcode::OpJump => {
                    let pos = BigEndian::read_u16(&ins[ip + 1..ip + 3]) as usize;
                    self.current_frame().ip = pos as i32 - 1;
                }
                Opcode::OpJumpNotTruthy => {
                    let pos = BigEndian::read_u16(&ins[ip + 1..ip + 3]) as usize;
                    self.current_frame().ip += 2;
                    let condition = self.pop();
                    if !condition.is_truthy() {
                        self.current_frame().ip = pos as i32 - 1;
                    }
                }
                Opcode::OpNull => {
                    self.push(Value::Null)?;
                }
                Opcode::OpGetGlobal => {
                    let global_index = BigEndian::read_u16(&ins[ip + 1..ip + 3]) as usize;
                    self.current_frame().ip += 2;
                    let val = self.globals[global_index].clone();
                    self.push(val)?;
                }
                Opcode::OpSetGlobal => {
                    let global_index = BigEndian::read_u16(&ins[ip + 1..ip + 3]) as usize;
                    self.current_frame().ip += 2;
                    self.globals[global_index] = self.pop();
                }
                Opcode::OpArray => {
                    let count = BigEndian::read_u16(&ins[ip + 1..ip + 3]) as usize;
                    self.current_frame().ip += 2;
                    let elements = self.build_array(self.sp - count, self.sp);
                    self.sp -= count;
                    self.push(Value::Object(Rc::new(Object::Array(elements))))?;
                }
                Opcode::OpHash => {
                    let count = BigEndian::read_u16(&ins[ip + 1..ip + 3]) as usize;
                    self.current_frame().ip += 2;
                    let elements = self.build_hash(self.sp - count, self.sp);
                    self.sp -= count;
                    self.push(Value::Object(Rc::new(Object::Hash(elements))))?;
                }
                Opcode::OpIndex => {
                    let index = self.pop();
                    let left = self.pop();
                    self.execute_index_operation(left, index)?;
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
                    self.push(Value::Null)?;
                }
                Opcode::OpCall => {
                    let num_args = ins[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    self.execute_call(num_args)?;
                }
                Opcode::OpTailCall => {
                    let num_args = ins[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    let base = self.current_frame().base_pointer;
                    let num_locals = self.current_frame().closure.func.num_locals;
                    for i in 0..num_args {
                        self.stack[base + i] = self.stack[self.sp - num_args + i].clone();
                    }
                    self.sp = base + num_locals;
                    self.current_frame().ip = -1;
                }
                Opcode::OpSetLocal => {
                    let local_index = ins[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    let base = self.current_frame().base_pointer;
                    self.stack[base + local_index] = self.pop();
                }
                Opcode::OpGetLocal => {
                    let local_index = ins[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    let base = self.current_frame().base_pointer;
                    let val = self.stack[base + local_index].clone();
                    self.push(val)?;
                }
                Opcode::OpGetBuiltin => {
                    let built_index = ins[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    let definition = BuiltIns.get(built_index).unwrap().1;
                    self.push(Value::Object(Rc::new(Object::Builtin(definition))))?;
                }
                Opcode::OpClosure => {
                    let const_index = BigEndian::read_u16(&ins[ip + 1..ip + 3]) as usize;
                    let num_free = ins[ip + 3] as usize;
                    self.current_frame().ip += 3;
                    self.push_closure(const_index, num_free)?;
                }
                Opcode::OpGetFree => {
                    let free_index = ins[ip + 1] as usize;
                    self.current_frame().ip += 1;
                    let current_closure = self.current_frame().closure.clone();
                    let val = Value::from_object(current_closure.free[free_index].clone());
                    self.push(val)?;
                }
                Opcode::OpCurrentClosure => {
                    let current_closure = self.current_frame().closure.clone();
                    self.push(Value::Object(Rc::new(Object::ClosureObj(current_closure))))?;
                }
            }
        }
        Ok(())
    }

    fn execute_binary_operation(&mut self, opcode: Opcode) -> Result<(), VMError> {
        let right = self.pop();
        let left = self.pop();
        match (&left, &right) {
            (Value::Integer(l), Value::Integer(r)) => {
                let result = match opcode {
                    Opcode::OpAdd => l + r,
                    Opcode::OpSub => l - r,
                    Opcode::OpMul => l * r,
                    Opcode::OpDiv => l / r,
                    Opcode::OpModulo => l % r,
                    _ => {
                        return Err(VMError::TypeError(format!(
                            "unknown integer operator: {:?}",
                            opcode
                        )));
                    }
                };
                self.push(Value::Integer(result))
            }
            (Value::Object(l), Value::Object(r)) => {
                if let (Object::String(ls), Object::String(rs)) = (&**l, &**r)
                    && opcode == Opcode::OpAdd
                {
                    let result = ls.to_string() + rs;
                    return self.push(Value::Object(Rc::new(Object::String(result))));
                }
                Err(VMError::TypeError(format!(
                    "unsupported binary operation {:?} for {} and {}",
                    opcode,
                    left.type_name(),
                    right.type_name()
                )))
            }
            _ => Err(VMError::TypeError(format!(
                "unsupported binary operation {:?} for {} and {}",
                opcode,
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    fn execute_comparison(&mut self, opcode: Opcode) -> Result<(), VMError> {
        let right = self.pop();
        let left = self.pop();
        match (&left, &right) {
            (Value::Integer(l), Value::Integer(r)) => {
                let result = match opcode {
                    Opcode::OpEqual => l == r,
                    Opcode::OpNotEqual => l != r,
                    Opcode::OpGreaterThan => l > r,
                    _ => {
                        return Err(VMError::TypeError(format!(
                            "unknown comparison operator: {:?}",
                            opcode
                        )));
                    }
                };
                self.push(Value::Boolean(result))
            }
            (Value::Boolean(l), Value::Boolean(r)) => {
                let result = match opcode {
                    Opcode::OpEqual => l == r,
                    Opcode::OpNotEqual => l != r,
                    _ => {
                        return Err(VMError::TypeError(format!(
                            "unknown boolean comparison operator: {:?}",
                            opcode
                        )));
                    }
                };
                self.push(Value::Boolean(result))
            }
            _ => Err(VMError::TypeError(format!(
                "unsupported comparison for {} and {}",
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    fn execute_minus_operation(&mut self) -> Result<(), VMError> {
        let operand = self.pop();
        match &operand {
            Value::Integer(l) => self.push(Value::Integer(-*l)),
            _ => Err(VMError::TypeError(format!(
                "unsupported negation for {}",
                operand.type_name()
            ))),
        }
    }

    fn execute_bang_operation(&mut self) -> Result<(), VMError> {
        let operand = self.pop();
        match &operand {
            Value::Boolean(l) => self.push(Value::Boolean(!*l)),
            _ => self.push(Value::Boolean(false)),
        }
    }

    pub fn last_popped_stack_elm(&self) -> Option<Value> {
        self.stack.get(self.sp).cloned()
    }

    fn pop(&mut self) -> Value {
        let o = self.stack[self.sp - 1].clone();
        self.sp -= 1;
        o
    }

    fn push(&mut self, v: Value) -> Result<(), VMError> {
        if self.sp >= STACK_SIZE {
            return Err(VMError::StackOverflow);
        }
        self.stack[self.sp] = v;
        self.sp += 1;
        Ok(())
    }

    fn build_array(&self, start: usize, end: usize) -> Vec<Rc<Object>> {
        let mut elements = Vec::with_capacity(end - start);
        for i in start..end {
            elements.push(self.stack[i].into_rc_object());
        }
        elements
    }

    fn build_hash(&self, start: usize, end: usize) -> HashMap<HashKey, Rc<Object>> {
        let mut elements = HashMap::new();
        for i in (start..end).step_by(2) {
            let key = self.stack[i].into_rc_object();
            let hash_key = HashKey::try_from(key.as_ref())
                .expect("compiler must only emit hashable keys in hash literals");
            let value = self.stack[i + 1].into_rc_object();
            elements.insert(hash_key, value);
        }
        elements
    }

    fn execute_index_operation(&mut self, left: Value, index: Value) -> Result<(), VMError> {
        match (&left, &index) {
            (Value::Object(o), Value::Integer(i)) => match &**o {
                Object::Array(arr) => self.execute_array_index(arr, *i),
                Object::Hash(hash) => self.execute_hash_index(hash, index.into_rc_object()),
                _ => Err(VMError::IndexError(format!(
                    "index operator not supported for {}",
                    left.type_name()
                ))),
            },
            (Value::Object(o), _) => match &**o {
                Object::Hash(hash) => self.execute_hash_index(hash, index.into_rc_object()),
                _ => Err(VMError::IndexError(format!(
                    "index operator not supported for {}",
                    left.type_name()
                ))),
            },
            _ => Err(VMError::IndexError(format!(
                "index operator not supported for {}",
                left.type_name()
            ))),
        }
    }

    fn execute_array_index(&mut self, array: &[Rc<Object>], index: i64) -> Result<(), VMError> {
        if index >= 0 && index < array.len() as i64 {
            self.push(Value::from_object(Rc::clone(&array[index as usize])))
        } else {
            self.push(Value::Null)
        }
    }

    fn execute_hash_index(
        &mut self,
        hash: &HashMap<HashKey, Rc<Object>>,
        index: Rc<Object>,
    ) -> Result<(), VMError> {
        match HashKey::try_from(index.as_ref()) {
            Ok(hash_key) => match hash.get(&hash_key) {
                Some(el) => self.push(Value::from_object(Rc::clone(el))),
                None => self.push(Value::Null),
            },
            Err(()) => Err(VMError::IndexError(format!(
                "index operator not supported for {}",
                index
            ))),
        }
    }

    fn current_frame(&mut self) -> &mut Frame {
        &mut self.frames[self.frame_index - 1]
    }

    fn push_frame(&mut self, frame: Frame) {
        self.frames[self.frame_index] = frame;
        self.frame_index += 1;
    }

    fn pop_frame(&mut self) -> Frame {
        self.frame_index -= 1;
        self.frames[self.frame_index].clone()
    }

    fn execute_call(&mut self, num_args: usize) -> Result<(), VMError> {
        let callee = self.stack[self.sp - 1 - num_args].clone();
        match &callee {
            Value::Object(o) => match &**o {
                Object::ClosureObj(cf) => self.call_closure(cf.clone(), num_args),
                Object::Builtin(bt) => self.call_builtin(*bt, num_args),
                _ => Err(VMError::NotCallable(callee.type_name().to_string())),
            },
            _ => Err(VMError::NotCallable(callee.type_name().to_string())),
        }
    }

    fn call_closure(&mut self, cl: Closure, num_args: usize) -> Result<(), VMError> {
        if cl.func.num_parameters != num_args {
            return Err(VMError::WrongArity {
                expected: cl.func.num_parameters,
                got: num_args,
            });
        }

        let frame = Frame::new(cl.clone(), self.sp - num_args);
        self.sp = frame.base_pointer + cl.func.num_locals;
        self.push_frame(frame);
        Ok(())
    }

    fn call_builtin(&mut self, bt: object::BuiltinFunc, num_args: usize) -> Result<(), VMError> {
        let args: Vec<Rc<Object>> = self.stack[self.sp - num_args..self.sp]
            .iter()
            .map(|v| v.into_rc_object())
            .collect();
        let result = bt(args);
        self.sp = self.sp - num_args - 1;
        self.push(Value::from_object(result))
    }

    fn push_closure(&mut self, const_index: usize, num_free: usize) -> Result<(), VMError> {
        let constant = self.constants[const_index].clone();
        match &constant {
            Value::Object(o) => match &**o {
                Object::CompiledFunction(f) => {
                    let mut free = Vec::with_capacity(num_free);
                    for i in 0..num_free {
                        free.push(self.stack[self.sp - num_free + i].into_rc_object());
                    }
                    self.sp -= num_free;
                    let closure = Object::ClosureObj(Closure {
                        func: f.clone(),
                        free,
                    });
                    self.push(Value::Object(Rc::new(closure)))
                }
                _ => Err(VMError::TypeError(format!(
                    "not a function: {}",
                    constant.type_name()
                ))),
            },
            _ => Err(VMError::TypeError(format!(
                "not a function: {}",
                constant.type_name()
            ))),
        }
    }
}
