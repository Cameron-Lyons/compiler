use std::collections::HashMap;
use once_cell::sync::Lazy;

pub type Opcode = u8;

pub const OPCONSTANT: Opcode = 1;

#[derive(Debug)]
pub struct Definition {
    pub name: &'static str,
    pub operand_widths: &'static [usize],
}

pub static DEFINITIONS: Lazy<HashMap<Opcode, Definition>> = Lazy::new(|| {
    let mut definitions = HashMap::new();

    definitions.insert(
        OPCONSTANT,
        Definition {
            name: "OpConstant",
            operand_widths: &[2],
        },
    );

    definitions
});


pub fn lookup(op: Opcode) -> Result<&'static Definition, String> {
    match DEFINITIONS.get(&op) {
        Some(def) => Ok(def),
        None => Err(format!("opcode {} undefined", op)),
    }
}


pub fn make(op: Opcode, operands: &[i32]) -> Result<Vec<u8>, String> {
    let def = lookup(op)?;

    let mut instruction = vec![op];

    for (i, operand) in operands.iter().enumerate() {
        let width = *def
            .operand_widths
            .get(i)
            .ok_or_else(|| "operand width not found".to_string())?;

        let bytes = match width {
            1 => {
                if *operand < 0 || *operand > 255 {
                    return Err(format!("Operand {} does not fit in 1 byte", operand));
                }
                vec![*operand as u8]
            }
            2 => {
                if *operand < 0 || *operand > 65535 {
                    return Err(format!("Operand {} does not fit in 2 bytes", operand));
                }
                (*operand as u16).to_be_bytes().to_vec()
            }
            4 => (*operand as u32).to_be_bytes().to_vec(),
            _ => return Err(format!("Unsupported operand width: {}", width)),
        };

        instruction.extend(bytes);
    }

    Ok(instruction)
}


