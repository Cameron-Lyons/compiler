use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use parser::ast::{BlockStatement, IDENTIFIER};

#[macro_use]
extern crate lazy_static;

use crate::environment::Env;

pub mod builtins;
pub mod environment;

pub type EvalError = String;
pub type BuiltinFunc = fn(Vec<Rc<Object>>) -> Rc<Object>;

/// Immutable key type for hash maps. Only hashable variants to satisfy clippy::mutable_key_type.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum HashKey {
    Integer(i64),
    Boolean(bool),
    String(String),
}

impl fmt::Display for HashKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            HashKey::Integer(i) => write!(f, "{}", i),
            HashKey::Boolean(b) => write!(f, "{}", b),
            HashKey::String(s) => write!(f, "{}", s),
        }
    }
}

impl TryFrom<&Object> for HashKey {
    type Error = ();

    fn try_from(obj: &Object) -> Result<Self, Self::Error> {
        match obj {
            Object::Integer(i) => Ok(HashKey::Integer(*i)),
            Object::Boolean(b) => Ok(HashKey::Boolean(*b)),
            Object::String(s) => Ok(HashKey::String(s.clone())),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    String(String),
    Array(Vec<Rc<Object>>),
    Hash(HashMap<HashKey, Rc<Object>>),
    Null,
    ReturnValue(Rc<Object>),
    Function(Vec<IDENTIFIER>, BlockStatement, Env),
    Builtin(BuiltinFunc),
    Error(String),
    CompiledFunction(Rc<CompiledFunction>),
    ClosureObj(Closure),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Integer(a), Object::Integer(b)) => a == b,
            (Object::Boolean(a), Object::Boolean(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            (Object::Array(a), Object::Array(b)) => a == b,
            (Object::Hash(a), Object::Hash(b)) => a == b,
            (Object::Null, Object::Null) => true,
            (Object::ReturnValue(a), Object::ReturnValue(b)) => a == b,
            (Object::Function(ap, ab, ae), Object::Function(bp, bb, be)) => {
                ap == bp && ab == bb && ae == be
            }
            (Object::Builtin(a), Object::Builtin(b)) => std::ptr::fn_addr_eq(*a, *b),
            (Object::Error(a), Object::Error(b)) => a == b,
            (Object::CompiledFunction(a), Object::CompiledFunction(b)) => a == b,
            (Object::ClosureObj(a), Object::ClosureObj(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Object::Integer(i) => write!(f, "{}", i),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::String(s) => write!(f, "{}", s),
            Object::Null => write!(f, "null"),
            Object::ReturnValue(expr) => write!(f, "{}", expr),
            Object::Function(params, body, _env) => {
                let func_params = params
                    .iter()
                    .map(|stmt| stmt.to_string())
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(f, "fn({}) {{ {} }}", func_params, body)
            }
            Object::Builtin(_) => write!(f, "[builtin function]"),
            Object::Error(e) => write!(f, "{}", e),
            Object::Array(e) => write!(
                f,
                "[{}]",
                e.iter()
                    .map(|o| o.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Object::Hash(map) => write!(
                f,
                "[{}]",
                map.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Object::CompiledFunction(_) => {
                write!(f, "[compiled function]")
            }
            Object::ClosureObj(_) => {
                write!(f, "[closure function]")
            }
        }
    }
}

impl Object {
    pub fn is_hashable(&self) -> bool {
        matches!(
            self,
            Object::Integer(_) | Object::Boolean(_) | Object::String(_)
        )
    }
}

impl Hash for Object {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Object::Integer(i) => i.hash(state),
            Object::Boolean(b) => b.hash(state),
            Object::String(s) => s.hash(state),
            t => panic!("can't hashable for {}", t),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompiledFunction {
    pub instructions: Vec<u8>,
    pub num_locals: usize,
    pub num_parameters: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Closure {
    pub func: Rc<CompiledFunction>,
    pub free: Vec<Rc<Object>>,
}
