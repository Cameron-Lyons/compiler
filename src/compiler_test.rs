#[cfg(test)]
mod tests {
    use crate::ast::parse_program;
    use crate::code::{make, Instructions, OPCONSTANT};
    use crate::compiler::{Bytecode, Compiler};
    use crate::object::Object;

    struct CompilerTestCase {
        input: String,
        expected_constants: Vec<Object>,
        expected_instructions: Vec<Instructions>,
    }

    #[test]
    fn test_integer_arithmetic() {
        let tests = vec![CompilerTestCase {
            input: "1 + 2".to_string(),
            expected_constants: vec![Object::Integer(1), Object::Integer(2)],
            expected_instructions: vec![
                make(OPCONSTANT, &[0]).unwrap(),
                make(OPCONSTANT, &[1]).unwrap(),
            ],
        }];

        run_compiler_tests(tests);
    }

    fn run_compiler_tests(tests: Vec<CompilerTestCase>) {
        for tt in tests {
            let program = parse_program(&tt.input).expect("Failed to parse input");

            let mut compiler = Compiler::new();
            match compiler.compile(program) {
                Ok(_) => (),
                Err(err) => panic!("Compiler error: {}", err),
            }

            let bytecode = compiler.bytecode();

            if let Err(err) = test_instructions(&tt.expected_instructions, &bytecode.instructions) {
                panic!("testInstructions failed: {}", err);
            }

            if let Err(err) = test_constants(&tt.expected_constants, &bytecode.constants) {
                panic!("testConstants failed: {}", err);
            }
        }
    }

    fn test_instructions(expected: &[Instructions], actual: &Instructions) -> Result<(), String> {
        let expected_flat: Instructions = expected.iter().flatten().cloned().collect();

        if expected_flat != *actual {
            return Err(format!(
                "Instructions do not match.\nExpected:\n{:?}\nActual:\n{:?}",
                expected_flat, actual
            ));
        }
        Ok(())
    }

    fn test_constants(expected: &[Object], actual: &[Object]) -> Result<(), String> {
        if expected.len() != actual.len() {
            return Err(format!(
                "Number of constants does not match.\nExpected: {}\nActual: {}",
                expected.len(),
                actual.len()
            ));
        }
        for (i, (exp, act)) in expected.iter().zip(actual.iter()).enumerate() {
            if exp != act {
                return Err(format!(
                    "Constant at index {} does not match.\nExpected: {:?}\nActual: {:?}",
                    i, exp, act
                ));
            }
        }
        Ok(())
    }
}
