use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Opcode {
    OpConstant = 0,
    OpPop,
    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    OpTrue,
    OpFalse,
    OpEqual,
    OpNotEqual,
    OpGreaterThan,
    OpMinus,
    OpBang,
    OpJumpNotTruthy,
    OpJump,
    OpNull,
    OpGetGlobal,
    OpSetGlobal,
    OpArray,
    OpHash,
    OpIndex,
    OpCall,
    OpReturnValue,
    OpReturn,
    OpGetLocal,
    OpSetLocal,
    OpGetBuiltin,
    OpClosure,
    OpGetFree,
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub name: &'static str,
    pub operand_widths: &'static [usize],
}

lazy_static::lazy_static! {
    pub static ref DEFINITIONS: HashMap<Opcode, Definition> = {
        use Opcode::*;
        let mut m = HashMap::new();

        m.insert(OpConstant,      Definition { name: "OpConstant",      operand_widths: &[2] });
        m.insert(OpPop,           Definition { name: "OpPop",           operand_widths: &[] });
        m.insert(OpAdd,           Definition { name: "OpAdd",           operand_widths: &[] });
        m.insert(OpSub,           Definition { name: "OpSub",           operand_widths: &[] });
        m.insert(OpMul,           Definition { name: "OpMul",           operand_widths: &[] });
        m.insert(OpDiv,           Definition { name: "OpDiv",           operand_widths: &[] });
        m.insert(OpTrue,          Definition { name: "OpTrue",          operand_widths: &[] });
        m.insert(OpFalse,         Definition { name: "OpFalse",         operand_widths: &[] });
        m.insert(OpEqual,         Definition { name: "OpEqual",         operand_widths: &[] });
        m.insert(OpNotEqual,      Definition { name: "OpNotEqual",      operand_widths: &[] });
        m.insert(OpGreaterThan,   Definition { name: "OpGreaterThan",   operand_widths: &[] });
        m.insert(OpMinus,         Definition { name: "OpMinus",         operand_widths: &[] });
        m.insert(OpBang,          Definition { name: "OpBang",          operand_widths: &[] });
        m.insert(OpJumpNotTruthy, Definition { name: "OpJumpNotTruthy", operand_widths: &[2] });
        m.insert(OpJump,          Definition { name: "OpJump",          operand_widths: &[2] });
        m.insert(OpNull,          Definition { name: "OpNull",          operand_widths: &[] });
        m.insert(OpGetGlobal,     Definition { name: "OpGetGlobal",     operand_widths: &[2] });
        m.insert(OpSetGlobal,     Definition { name: "OpSetGlobal",     operand_widths: &[2] });
        m.insert(OpArray,         Definition { name: "OpArray",         operand_widths: &[2] });
        m.insert(OpHash,          Definition { name: "OpHash",          operand_widths: &[2] });
        m.insert(OpIndex,         Definition { name: "OpIndex",         operand_widths: &[] });
        m.insert(OpCall,          Definition { name: "OpCall",          operand_widths: &[1] });
        m.insert(OpReturnValue,   Definition { name: "OpReturnValue",   operand_widths: &[] });
        m.insert(OpReturn,        Definition { name: "OpReturn",        operand_widths: &[] });
        m.insert(OpGetLocal,      Definition { name: "OpGetLocal",      operand_widths: &[1] });
        m.insert(OpSetLocal,      Definition { name: "OpSetLocal",      operand_widths: &[1] });
        m.insert(OpGetBuiltin,    Definition { name: "OpGetBuiltin",    operand_widths: &[1] });
        m.insert(OpClosure,       Definition { name: "OpClosure",       operand_widths: &[2, 1] });
        m.insert(OpGetFree,       Definition { name: "OpGetFree",       operand_widths: &[1] });

        m
    };
}

pub fn opcode_from_u8(b: u8) -> Option<Opcode> {
    if b <= Opcode::OpGetFree as u8 {
        Some(unsafe { std::mem::transmute::<u8, Opcode>(b) })
    } else {
        None
    }
}

pub fn lookup(op: u8) -> Result<&'static Definition, String> {
    if let Some(o) = opcode_from_u8(op) {
        if let Some(def) = DEFINITIONS.get(&o) {
            Ok(def)
        } else {
            Err(format!("opcode {} undefined in DEFINITIONS", op))
        }
    } else {
        Err(format!("opcode {} invalid (out of range)", op))
    }
}

pub fn make(op: Opcode, operands: &[usize]) -> Vec<u8> {
    let def = match DEFINITIONS.get(&op) {
        Some(d) => d,
        None => {
            return vec![];
        }
    };

    let mut instruction_len = 1;
    for &w in def.operand_widths {
        instruction_len += w;
    }

    let mut instruction = vec![0u8; instruction_len];
    instruction[0] = op as u8;

    let mut offset = 1;
    for (i, &operand) in operands.iter().enumerate() {
        let width = def.operand_widths[i];
        match width {
            2 => {
                // Write a big-endian u16
                let val = operand as u16;
                instruction[offset] = (val >> 8) as u8;
                instruction[offset + 1] = (val & 0xff) as u8;
            }
            1 => {
                instruction[offset] = operand as u8;
            }
            _ => {}
        }
        offset += width;
    }

    instruction
}

pub fn read_operands(def: &Definition, ins: &[u8]) -> (Vec<usize>, usize) {
    let mut operands = vec![0usize; def.operand_widths.len()];
    let mut offset = 0;

    for (i, &width) in def.operand_widths.iter().enumerate() {
        match width {
            2 => {
                let big = ((ins[offset] as u16) << 8) | (ins[offset + 1] as u16);
                operands[i] = big as usize;
            }
            1 => {
                operands[i] = ins[offset] as usize;
            }
            _ => {}
        }
        offset += width;
    }

    (operands, offset)
}

#[derive(Clone)]
pub struct Instructions(pub Vec<u8>);

impl Instructions {
    pub fn read_uint16(ins: &[u8]) -> u16 {
        ((ins[0] as u16) << 8) | (ins[1] as u16)
    }

    pub fn read_uint8(ins: &[u8]) -> u8 {
        ins[0]
    }

    fn fmt_instruction(&self, def: &Definition, operands: &[usize]) -> String {
        let operand_count = def.operand_widths.len();
        if operands.len() != operand_count {
            return format!(
                "ERROR: operand len {} does not match defined {}",
                operands.len(),
                operand_count
            );
        }

        match operand_count {
            0 => format!("{}", def.name),
            1 => format!("{} {}", def.name, operands[0]),
            2 => format!("{} {} {}", def.name, operands[0], operands[1]),
            _ => format!("ERROR: unhandled operandCount for {}", def.name),
        }
    }
}

impl fmt::Display for Instructions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ins = &self.0;
        let mut i = 0;

        while i < ins.len() {
            match lookup(ins[i]) {
                Ok(def) => {
                    let (operands, read) = read_operands(def, &ins[i + 1..]);
                    let line = format!("{:04} {}\n", i, self.fmt_instruction(def, &operands));
                    write!(f, "{}", line)?;
                    i += 1 + read;
                }
                Err(e) => {
                    let line = format!("ERROR: {}\n", e);
                    write!(f, "{}", line)?;
                    i += 1;
                }
            }
        }
        Ok(())
    }
}
