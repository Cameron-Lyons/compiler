use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fmt::{self, Write};

pub type Opcode = u8;

pub const OPCONSTANT: Opcode = 1;
pub const OPADD: Opcode = 2;
pub const OPPOP: Opcode = 3;

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
            operand_widths: &[2], // Operand is a 16-bit integer
        },
    );

    definitions.insert(
        OPADD,
        Definition {
            name: "OpAdd",
            operand_widths: &[], // No operands for OpAdd
        },
    );

    definitions.insert(
        OPPOP,
        Definition {
            name: "OpPop",
            operand_widths: &[],
        },
    );

    definitions
});

pub fn lookup(op: Opcode) -> Result<&'static Definition, String> {
    DEFINITIONS
        .get(&op)
        .ok_or_else(|| format!("opcode {} undefined", op))
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
            2 => operand.to_be_bytes()[6..8].to_vec(), // Extract last 2 bytes for 16-bit
            _ => return Err(format!("unsupported operand width: {}", width)),
        };

        instruction.extend(bytes);
    }

    Ok(instruction)
}
#[derive(Debug, Clone)]
pub struct Instructions(pub Vec<u8>);

impl Instructions {
    pub fn new(bytes: Vec<u8>) -> Self {
        Instructions(bytes)
    }

    pub fn to_pretty_string(&self) -> String {
        let mut out = String::new();

        let mut i = 0;
        while i < self.0.len() {
            let op = self.0[i];
            match lookup(op) {
                Ok(def) => {
                    let (operands, read) = read_operands(def, &self.0[i + 1..]);
                    let instr_str = self.fmt_instruction(def, &operands);
                    let _ = writeln!(&mut out, "{:04} {}", i, instr_str);
                    i += 1 + read;
                }
                Err(err) => {
                    let _ = writeln!(&mut out, "ERROR: {}", err);
                    i += 1;
                }
            }
        }

        out
    }

    fn fmt_instruction(&self, def: &Definition, operands: &[i32]) -> String {
        let operand_count = def.operand_widths.len();

        if operands.len() != operand_count {
            return format!(
                "ERROR: operand len {} does not match defined {}\n",
                operands.len(),
                operand_count
            );
        }

        match operand_count {
            1 => format!("{} {}", def.name, operands[0]),
            _ => format!("ERROR: unhandled operandCount for {}\n", def.name),
        }
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

fn read_operands(def: &Definition, ins: &[u8]) -> (Vec<i32>, usize) {
    let mut operands = Vec::new();
    let mut offset = 0;
    for &width in def.operand_widths {
        let operand = match width {
            1 => {
                let val = ins[offset] as i32;
                offset += 1;
                val
            }
            2 => {
                let val = ((ins[offset] as u16) << 8) | ins[offset + 1] as u16;
                offset += 2;
                val as i32
            }
            4 => {
                let val = ((ins[offset] as u32) << 24)
                    | ((ins[offset + 1] as u32) << 16)
                    | ((ins[offset + 2] as u32) << 8)
                    | (ins[offset + 3] as u32);
                offset += 4;
                val as i32
            }
            _ => unreachable!("Unsupported operand width"),
        };
        operands.push(operand);
    }
    (operands, offset)
}

impl fmt::Display for Instructions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.to_pretty_string();
        write!(f, "{}", s)
    }
}
