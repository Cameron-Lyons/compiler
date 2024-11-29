#[derive(Debug)]
struct Definition {
    name: &'static str,
    operand_widths: &'static [usize],
}

static DEFINITIONS: &'static [(u8, Definition)] = &[(
    1, // Example Opcode for OpConstant
    Definition {
        name: "OpConstant",
        operand_widths: &[2],
    },
)];

fn lookup(op: u8) -> Result<&'static Definition, String> {
    // Attempt to find the opcode definition
    for (opcode, def) in DEFINITIONS {
        if *opcode == op {
            return Ok(def);
        }
    }
    Err(format!("opcode {} undefined", op))
}

fn main() {
    // Test the lookup function
    match lookup(1) {
        Ok(def) => println!("Found definition: {:?}", def),
        Err(err) => println!("Error: {}", err),
    }

    match lookup(2) {
        Ok(def) => println!("Found definition: {:?}", def),
        Err(err) => println!("Error: {}", err),
    }
}
