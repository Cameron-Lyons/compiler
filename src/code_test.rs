#[cfg(test)]
mod tests {
    use crate::code::*;

    #[test]
    fn test_make() {
        struct TestCase {
            op: Opcode,
            operands: Vec<i32>,
            expected: Vec<u8>,
        }

        // Assuming OPCONSTANT expects a 2-byte operand,
        // and OPADD expects no operands (just the opcode).
        // Adjust these according to your actual opcode definitions.
        let tests = vec![
            TestCase {
                op: OPCONSTANT,
                operands: vec![65534],
                expected: vec![OPCONSTANT, 255, 254],
            },
            TestCase {
                op: OPCONSTANT,
                operands: vec![1],
                expected: vec![OPCONSTANT, 0, 1],
            },
            // A test case with multiple operands for an opcode that requires them
            // Assuming an opcode that expects two 2-byte operands (e.g., OPGETLOCAL).
            // Example: If OPGETLOCAL expects two 2-byte operands, we might have:
            // OPGETLOCAL, 2-byte operand (200), 2-byte operand (300)
            // This would result in: [OPGETLOCAL, 0, 200, 1, 44] (since 300 = 0x012C)
            // Adjust accordingly to match your opcode definitions.
            TestCase {
                op: OPGETLOCAL,
                operands: vec![200, 300],
                expected: vec![OPGETLOCAL, 0, 200, 1, 44],
            },
        ];

        for tt in tests {
            let instruction = make(tt.op, &tt.operands).expect("make function failed");

            assert_eq!(
                instruction.len(),
                tt.expected.len(),
                "instruction has wrong length. want={}, got={}",
                tt.expected.len(),
                instruction.len()
            );

            for (i, byte) in tt.expected.iter().enumerate() {
                assert_eq!(
                    instruction[i], *byte,
                    "wrong byte at pos {}. want={}, got={}",
                    i, byte, instruction[i]
                );
            }
        }
    }

    #[test]
    fn test_make_with_invalid_operand() {
        // Operand exceeds the allowed width (2 bytes -> max 65535)
        let result = make(OPCONSTANT, &[70000]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Operand 70000 does not fit in 2 bytes");
    }

    #[test]
    fn test_lookup_defined_opcode() {
        let def = lookup(OPCONSTANT);
        assert!(def.is_ok());
        let definition = def.unwrap();
        assert_eq!(definition.name, "OpConstant");
        assert_eq!(definition.operand_widths, &[2]);

        // Test another known opcode, e.g., OPADD which might have no operands.
        let def_add = lookup(OPADD);
        assert!(def_add.is_ok());
        let definition_add = def_add.unwrap();
        assert_eq!(definition_add.name, "OpAdd");
        assert_eq!(definition_add.operand_widths, &[]);
    }

    #[test]
    fn test_lookup_undefined_opcode() {
        // Use a random opcode number that doesn't exist
        let def = lookup(0xFF);
        assert!(def.is_err());
        assert_eq!(def.unwrap_err(), "Opcode 255 undefined");
    }
}
