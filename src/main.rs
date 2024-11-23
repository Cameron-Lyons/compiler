use regex::Regex;
use std::env;
use std::fs;
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <source_file>", args[0]);
        return Ok(());
    }

    let source_file = &args[1];
    let assembly_file = match create_assembly_file_name(source_file) {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Error creating assembly file name: {}", e);
            return Ok(());
        }
    };

    let source_re = r"int main\s*\(\s*\)\s*{\s*return\s+(?P<ret>[0-9]+)\s*;\s*}";

    let assembly_format = r".globl _main
_main:
    movl    ${}, %eax
    ret
";

    let mut infile = match fs::File::open(source_file) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening source file: {}", e);
            return Ok(());
        }
    };
    let mut source_data = String::new();
    infile.read_to_string(&mut source_data)?;

    let source = source_data.trim();

    let re = match Regex::new(source_re) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error compiling regex: {}", e);
            return Ok(());
        }
    };

    if let Some(caps) = re.captures(source) {
        if let Some(ret_val) = caps.name("ret") {
            let ret_val_trimmed = ret_val.as_str().trim();
            let final_val: i32 = match ret_val_trimmed.parse() {
                Ok(val) => val,
                Err(_) => {
                    eprintln!("Failed to parse return value");
                    return Ok(());
                }
            };

            let mut outfile = match fs::File::create(&assembly_file) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Error creating output file: {}", e);
                    return Ok(());
                }
            };
            let assembly_code = format!("{} {}", assembly_format, final_val);
            outfile.write_all(assembly_code.as_bytes())?;
        } else {
            eprintln!("Failed to extract return value");
        }
    } else {
        eprintln!("No match found in source file");
    }

    Ok(())
}

fn create_assembly_file_name(source_file: &str) -> io::Result<String> {
    if let Some(index) = source_file.rfind('.') {
        let base_name = &source_file[..index];
        Ok(format!("{}.s", base_name))
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid file name",
        ))
    }
}
