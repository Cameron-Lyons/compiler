
use std::collections::HashMap;
use once_cell::sync::Lazy;

#[derive(Debug)]
pub struct Definition {
    pub name: &'static str,
    pub operand_widths: &'static [usize],
}

pub static DEFINITIONS: Lazy<HashMap<u8, Definition>> = Lazy::new(|| {
    let mut definitions = HashMap::new();

    definitions.insert(
        1,
        Definition {
            name: "OpConstant",
            operand_widths: &[2],
        },
    );

    definitions
});

pub fn lookup(op: u8) -> Result<&'static Definition, String> {
    match DEFINITIONS.get(&op) {
        Some(def) => Ok(def),
        None => Err(format!("opcode {} undefined", op)),
    }
}

