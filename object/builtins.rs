use crate::{BuiltinFunc, Object};
use std::rc::Rc;

lazy_static! {
    pub static ref BuiltIns: Vec<(&'static str, BuiltinFunc)> = vec![
        ("len", len),
        ("puts", puts),
        ("first", first),
        ("last", last),
        ("rest", rest),
        ("push", push),
        ("print", puts)
    ];
}

fn wrong_arity(name: &str, expected: usize, got: usize) -> Rc<Object> {
    Rc::new(Object::Error(format!(
        "builtin {} expected {} argument{}, got {}",
        name,
        expected,
        if expected == 1 { "" } else { "s" },
        got
    )))
}

pub fn len(args: Vec<Rc<Object>>) -> Rc<Object> {
    if args.len() != 1 {
        return wrong_arity("len", 1, args.len());
    }
    Rc::from(match &*args[0] {
        Object::String(s) => Object::Integer(s.len() as i64),
        Object::Array(a) => Object::Integer(a.len() as i64),
        o => Object::Error(format!("builtin len not supported for type {}", o)),
    })
}

pub fn puts(args: Vec<Rc<Object>>) -> Rc<Object> {
    args.iter().for_each(|obj| println!("{}", obj));
    Rc::from(Object::Null)
}

pub fn first(args: Vec<Rc<Object>>) -> Rc<Object> {
    if args.len() != 1 {
        return wrong_arity("first", 1, args.len());
    }

    match &*args[0] {
        Object::Array(s) => match s.first() {
            Some(obj) => Rc::clone(obj),
            None => Rc::new(Object::Null),
        },
        o => Rc::new(Object::Error(format!(
            "builtin first not supported for type {}",
            o
        ))),
    }
}

pub fn last(args: Vec<Rc<Object>>) -> Rc<Object> {
    if args.len() != 1 {
        return wrong_arity("last", 1, args.len());
    }

    match &*args[0] {
        Object::Array(s) => match s.last() {
            Some(obj) => Rc::clone(obj),
            None => Rc::new(Object::Null),
        },
        o => Rc::new(Object::Error(format!(
            "builtin last not supported for type {}",
            o
        ))),
    }
}

pub fn rest(args: Vec<Rc<Object>>) -> Rc<Object> {
    if args.len() != 1 {
        return wrong_arity("rest", 1, args.len());
    }

    match &*args[0] {
        Object::Array(s) => {
            let len = s.len();
            if len > 0 {
                let new_array = s[1..len].to_vec();
                return Rc::new(Object::Array(new_array));
            }
            Rc::new(Object::Null)
        }
        o => Rc::new(Object::Error(format!(
            "builtin rest not supported for type {}",
            o
        ))),
    }
}

pub fn push(args: Vec<Rc<Object>>) -> Rc<Object> {
    if args.len() != 2 {
        return wrong_arity("push", 2, args.len());
    }

    let array = &args[0];
    let obj = Rc::clone(&args[1]);
    match &**array {
        Object::Array(s) => {
            let mut new_array = s.clone();
            new_array.push(obj);
            Rc::new(Object::Array(new_array))
        }
        o => Rc::new(Object::Error(format!(
            "builtin push not supported for type {}",
            o
        ))),
    }
}
