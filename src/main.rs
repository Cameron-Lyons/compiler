mod code;

#[cfg(test)]
mod code_test;

fn main() {
    match code::lookup(1) {
        Ok(def) => println!("Found definition: {:?}", def),
        Err(err) => println!("Error: {}", err),
    }

    match code::lookup(2) {
        Ok(def) => println!("Found definition: {:?}", def),
        Err(err) => println!("Error: {}", err),
    }

    match code::make(code::OPCONSTANT, &[65534]) {
        Ok(instruction) => println!("Instruction: {:?}", instruction),
        Err(err) => println!("Error: {}", err),
    }
}
