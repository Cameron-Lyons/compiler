use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const LEFT_BRACE: &str = "{";
const RIGHT_BRACE: &str = "}";
const COMMA: &str = ",";
const COLON: &str = ":";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    IntegerObj,
    BooleanObj,
    NullObj,
    ReturnValueObj,
    ErrorObj,
    FunctionObj,
    StringObj,
    BuiltinObj,
    ArrayObj,
    HashObj,
    CompiledFunctionObj,
    ClosureObj,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HashKey {
    pub obj_type: ObjectType,
    pub value: u64,
}

pub trait Hashable {
    fn to_hash_key(&self) -> HashKey;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Integer(Integer),
    Boolean(Boolean),
    Null(Null),
    ReturnValue(ReturnValue),
    Error(Error),
    Function(Function),
    String(StringObj),
    Builtin(Builtin),
    Array(Array),
    Hash(HashObj),
    CompiledFunction(CompiledFunction),
    Closure(Closure),
}

impl Object {
    pub fn object_type(&self) -> ObjectType {
        match self {
            Object::Integer(_) => ObjectType::IntegerObj,
            Object::Boolean(_) => ObjectType::BooleanObj,
            Object::Null(_) => ObjectType::NullObj,
            Object::ReturnValue(_) => ObjectType::ReturnValueObj,
            Object::Error(_) => ObjectType::ErrorObj,
            Object::Function(_) => ObjectType::FunctionObj,
            Object::String(_) => ObjectType::StringObj,
            Object::Builtin(_) => ObjectType::BuiltinObj,
            Object::Array(_) => ObjectType::ArrayObj,
            Object::Hash(_) => ObjectType::HashObj,
            Object::CompiledFunction(_) => ObjectType::CompiledFunctionObj,
            Object::Closure(_) => ObjectType::ClosureObj,
        }
    }

    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(i) => i.inspect(),
            Object::Boolean(b) => b.inspect(),
            Object::Null(n) => n.inspect(),
            Object::ReturnValue(rv) => rv.inspect(),
            Object::Error(e) => e.inspect(),
            Object::Function(f) => f.inspect(),
            Object::String(s) => s.inspect(),
            Object::Builtin(b) => b.inspect(),
            Object::Array(a) => a.inspect(),
            Object::Hash(h) => h.inspect(),
            Object::CompiledFunction(cf) => cf.inspect(),
            Object::Closure(c) => c.inspect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Integer {
    pub value: i64,
}

impl Integer {
    pub fn inspect(&self) -> String {
        format!("{}", self.value)
    }
}

impl Hashable for Integer {
    fn to_hash_key(&self) -> HashKey {
        HashKey {
            obj_type: ObjectType::IntegerObj,
            value: self.value as u64,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Boolean {
    pub value: bool,
}

impl Boolean {
    pub fn inspect(&self) -> String {
        format!("{}", self.value)
    }
}

impl Hashable for Boolean {
    fn to_hash_key(&self) -> HashKey {
        HashKey {
            obj_type: ObjectType::BooleanObj,
            value: if self.value { 1 } else { 0 },
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Null;

impl Null {
    pub fn inspect(&self) -> String {
        "null".to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReturnValue {
    pub value: Box<Object>,
}

impl ReturnValue {
    pub fn inspect(&self) -> String {
        self.value.inspect()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn inspect(&self) -> String {
        format!("ERROR: {}", self.message)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub parameters: Vec<crate::ast::Identifier>, // placeholders
    pub body: Box<crate::ast::BlockStatement>,   // placeholders
                                                 // Env is omitted or replaced with a reference to your environment type if needed
}

impl Function {
    pub fn inspect(&self) -> String {
        let mut out = String::new();

        out.push_str("fn(");
        let params: Vec<String> = self.parameters.iter().map(|p| p.to_string()).collect();
        out.push_str(&params.join(", "));
        out.push_str(") {\n");
        out.push_str(&self.body.to_string());
        out.push_str("\n}");

        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringObj {
    pub value: String,
}

impl StringObj {
    pub fn inspect(&self) -> String {
        self.value.clone()
    }
}

impl Hashable for StringObj {
    fn to_hash_key(&self) -> HashKey {
        let mut hasher = fnv::FnvHasher::default();
        hasher.write(self.value.as_bytes());
        HashKey {
            obj_type: ObjectType::StringObj,
            value: hasher.finish(),
        }
    }
}

pub type BuiltinFunction = fn(&[Object]) -> Object;

#[derive(Debug, Clone)]
pub struct Builtin {
    pub func: BuiltinFunction,
}

impl PartialEq for Builtin {
    fn eq(&self, _other: &Builtin) -> bool {
        false
    }
}

impl Builtin {
    pub fn inspect(&self) -> String {
        "builtin function".to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Array {
    pub elements: Vec<Object>,
}

impl Array {
    pub fn inspect(&self) -> String {
        let mut out = String::new();
        out.push('[');
        let elem_strs: Vec<String> = self.elements.iter().map(|e| e.inspect()).collect();
        out.push_str(&elem_strs.join(", "));
        out.push(']');
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HashPair {
    pub key: Object,
    pub value: Object,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HashObj {
    pub pairs: HashMap<HashKey, HashPair>,
}

impl HashObj {
    pub fn inspect(&self) -> String {
        let mut out = String::new();
        let mut pair_strs = vec![];

        for (_, pair) in &self.pairs {
            let s = format!("{}: {}", pair.key.inspect(), pair.value.inspect());
            pair_strs.push(s);
        }
        out.push_str(LEFT_BRACE);
        out.push_str(&pair_strs.join(&format!("{} ", COMMA)));
        out.push_str(RIGHT_BRACE);
        out
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledFunction {
    pub instructions: crate::code::Instructions,
    pub num_locals: usize,
    pub num_parameters: usize,
}

impl CompiledFunction {
    pub fn inspect(&self) -> String {
        format!("CompiledFunction[{:p}]", self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub fn_obj: Box<CompiledFunction>,
    pub free: Vec<Object>,
}

impl Closure {
    pub fn inspect(&self) -> String {
        format!("Closure[{:p}]", self)
    }
}
