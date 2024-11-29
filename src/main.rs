mod code;

fn main() {
    // Test the lookup function
    match code::lookup(1) {
        Ok(def) => println!("Found definition: {:?}", def),
        Err(err) => println!("Error: {}", err),
    }

    match code::lookup(2) {
        Ok(def) => println!("Found definition: {:?}", def),
        Err(err) => println!("Error: {}", err),
    }
}
