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
    }
}
