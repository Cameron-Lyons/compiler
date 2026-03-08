use crate::compiler::Compiler;
use crate::compiler_test::test_constants;
use crate::vm::{VM, VMError};
use object::Object;
use parser::parse;

pub struct VmTestCase<'a> {
    pub(crate) input: &'a str,
    pub(crate) expected: Object,
}

pub fn run_vm_tests(tests: Vec<VmTestCase>) {
    for t in tests {
        let program = parse(t.input).unwrap();
        let mut compiler = Compiler::new();
        let bytecodes = compiler.compile(&program).unwrap();
        println!(
            "ins {} for input {}",
            bytecodes.instructions.string().unwrap(),
            t.input
        );
        let mut vm = VM::new(bytecodes);
        vm.run().unwrap();
        let got = vm.last_popped_stack_elm().unwrap().into_rc_object();
        let expected_argument = t.expected;
        test_constants(&[expected_argument], &[got]);
    }
}

pub fn run_vm_error_test(input: &str) -> VMError {
    let program = parse(input).unwrap();
    let mut compiler = Compiler::new();
    let bytecodes = compiler.compile(&program).unwrap();
    let mut vm = VM::new(bytecodes);
    vm.run().unwrap_err()
}

#[cfg(test)]
mod tests {
    use crate::compiler::Bytecode;
    use crate::op_code::Opcode::{OpClosure, OpGetBuiltin, OpGreaterThan, OpSub};
    use crate::op_code::{Instructions, OpCodeError, make_instructions};
    use crate::vm::{VM, VMError};
    use crate::vm_test::{VmTestCase, run_vm_error_test, run_vm_tests};
    use object::{HashKey, Object};
    use std::collections::HashMap;
    use std::rc::Rc;

    #[test]
    fn test_integer_arithmetic() {
        let tests: Vec<VmTestCase> = vec![
            VmTestCase {
                input: "1",
                expected: Object::Integer(1),
            },
            VmTestCase {
                input: "2",
                expected: Object::Integer(2),
            },
            VmTestCase {
                input: "1 + 2",
                expected: Object::Integer(3),
            },
            VmTestCase {
                input: "4 / 2",
                expected: Object::Integer(2),
            },
            VmTestCase {
                input: "50 / 2 * 2 + 10 - 5",
                expected: Object::Integer(55),
            },
            VmTestCase {
                input: "5 * (2 + 10)",
                expected: Object::Integer(60),
            },
            VmTestCase {
                input: "5 + 5 + 5 + 5 - 10",
                expected: Object::Integer(10),
            },
            VmTestCase {
                input: "2 * 2 * 2 * 2 * 2",
                expected: Object::Integer(32),
            },
            VmTestCase {
                input: "5 * 2 + 10",
                expected: Object::Integer(20),
            },
            VmTestCase {
                input: "5 + 2 * 10",
                expected: Object::Integer(25),
            },
            VmTestCase {
                input: "5 * (2 + 10)",
                expected: Object::Integer(60),
            },
            VmTestCase {
                input: "-5",
                expected: Object::Integer(-5),
            },
            VmTestCase {
                input: "-10",
                expected: Object::Integer(-10),
            },
            VmTestCase {
                input: "-50 + 100 + -50",
                expected: Object::Integer(0),
            },
            VmTestCase {
                input: "(5 + 10 * 2 + 15 / 3) * 2 + -10",
                expected: Object::Integer(50),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn test_boolean_expressions() {
        let tests: Vec<VmTestCase> = vec![
            VmTestCase {
                input: "true",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "false",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "true",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "false",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "1 < 2",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "1 > 2",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "1 < 1",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "1 > 1",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "1 == 1",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "1 != 1",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "1 == 2",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "1 != 2",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "true == true",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "false == false",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "true == false",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "true != false",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "false != true",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "(1 < 2) == true",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "(1 < 2) == false",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "(1 > 2) == true",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "(1 > 2) == false",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "!true",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "!false",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "!5",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "!!true",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "!!false",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "!!5",
                expected: Object::Boolean(true),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn test_runtime_error_variants_from_source() {
        assert!(matches!(
            run_vm_error_test("fn(a) { a }();"),
            VMError::WrongArity {
                expected: 1,
                got: 0,
            }
        ));
        assert!(matches!(
            run_vm_error_test("1();"),
            VMError::NotCallable("INTEGER")
        ));
        assert!(matches!(
            run_vm_error_test("-true;"),
            VMError::UnsupportedNegation("BOOLEAN")
        ));
        assert!(matches!(
            run_vm_error_test("1 - true;"),
            VMError::UnsupportedBinaryOperation {
                op: OpSub,
                left: "INTEGER",
                right: "BOOLEAN",
            }
        ));
        assert!(matches!(
            run_vm_error_test("1 == true;"),
            VMError::UnsupportedComparison {
                left: "INTEGER",
                right: "BOOLEAN",
            }
        ));
        assert!(matches!(
            run_vm_error_test("true > false;"),
            VMError::UnknownBooleanComparisonOperator(OpGreaterThan)
        ));
        assert!(matches!(
            run_vm_error_test("1[0];"),
            VMError::UnsupportedIndexOperator("INTEGER")
        ));
        assert!(matches!(
            run_vm_error_test("{[1]: 2};"),
            VMError::UnusableAsHashKey(key) if key == "[1]"
        ));
    }

    #[test]
    fn test_runtime_error_variants_from_invalid_bytecode() {
        let mut invalid_opcode_vm = VM::new(Bytecode {
            instructions: Instructions { bytes: vec![255] },
            constants: vec![],
        });
        assert!(matches!(
            invalid_opcode_vm.run().unwrap_err(),
            VMError::Opcode(OpCodeError::InvalidOpcodeByte {
                byte: 255,
                position: None,
            })
        ));

        let mut invalid_builtin_vm = VM::new(Bytecode {
            instructions: make_instructions(OpGetBuiltin, &[255]),
            constants: vec![],
        });
        assert!(matches!(
            invalid_builtin_vm.run().unwrap_err(),
            VMError::UnknownBuiltinIndex(255)
        ));

        let mut invalid_closure_vm = VM::new(Bytecode {
            instructions: make_instructions(OpClosure, &[0, 0]),
            constants: vec![Rc::new(Object::Integer(1))],
        });
        assert!(matches!(
            invalid_closure_vm.run().unwrap_err(),
            VMError::ExpectedCompiledFunction("INTEGER")
        ));
    }

    #[test]
    fn test_conditionals() {
        let tests = vec![
            VmTestCase {
                input: "if (true) { 10 }",
                expected: Object::Integer(10),
            },
            VmTestCase {
                input: "if (true) { 10 } else { 20 }",
                expected: Object::Integer(10),
            },
            VmTestCase {
                input: "if (false) { 10 } else { 20 }",
                expected: Object::Integer(20),
            },
            VmTestCase {
                input: "if (1) { 10 }",
                expected: Object::Integer(10),
            },
            VmTestCase {
                input: "if (1 < 2) { 10 }",
                expected: Object::Integer(10),
            },
            VmTestCase {
                input: "if (1 < 2) { 10 } else { 20 }",
                expected: Object::Integer(10),
            },
            VmTestCase {
                input: "if (1 > 2) { 10 } else { 20 }",
                expected: Object::Integer(20),
            },
            VmTestCase {
                input: "if (1 > 2) { 10 }",
                expected: Object::Null,
            },
            VmTestCase {
                input: "if (false) { 10 }",
                expected: Object::Null,
            },
            VmTestCase {
                input: "if ((if (false) { 10 })) { 10 } else { 20 }",
                expected: Object::Integer(20),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn test_global_let_statements() {
        let tests = vec![
            VmTestCase {
                input: "let one = 1; one",
                expected: Object::Integer(1),
            },
            VmTestCase {
                input: "let one = 1; let two = 2; one + two",
                expected: Object::Integer(3),
            },
            VmTestCase {
                input: "let one = 1; let two = one + one; one + two",
                expected: Object::Integer(3),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn test_strings() {
        let tests = vec![
            VmTestCase {
                input: "\"monkey\"",
                expected: Object::String("monkey".to_string()),
            },
            VmTestCase {
                input: "\"mon\" + \"key\"",
                expected: Object::String("monkey".to_string()),
            },
            VmTestCase {
                input: "\"mon\" + \"key\" + \"banana\"",
                expected: Object::String("monkeybanana".to_string()),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn test_arrays() {
        fn map_vec_to_object(vec: Vec<i64>) -> Object {
            let array = vec
                .iter()
                .map(|i| Rc::new(Object::Integer(*i)))
                .collect::<Vec<Rc<Object>>>();
            Object::Array(array)
        }
        let tests = vec![
            VmTestCase {
                input: "[]",
                expected: map_vec_to_object(vec![]),
            },
            VmTestCase {
                input: "[1, 2, 3]",
                expected: map_vec_to_object(vec![1, 2, 3]),
            },
            VmTestCase {
                input: "[1 + 2, 3 * 4, 5 + 6]",
                expected: map_vec_to_object(vec![3, 12, 11]),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn test_hash() {
        fn map_vec_to_object(vec: Vec<(i64, i64)>) -> Object {
            let hash = vec.iter().fold(HashMap::new(), |mut acc, (k, v)| {
                acc.insert(HashKey::Integer(*k), Rc::new(Object::Integer(*v)));
                acc
            });
            Object::Hash(hash)
        }
        let tests = vec![
            VmTestCase {
                input: "{}",
                expected: Object::Hash(HashMap::new()),
            },
            VmTestCase {
                input: "{1: 2, 2: 3}",
                expected: map_vec_to_object(vec![(1, 2), (2, 3)]),
            },
            VmTestCase {
                input: "{1 + 1: 2 * 2, 3 + 3: 4 * 4}",
                expected: map_vec_to_object(vec![(2, 4), (6, 16)]),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn test_index() {
        let tests = vec![
            VmTestCase {
                input: "[1, 2, 3][1]",
                expected: Object::Integer(2),
            },
            VmTestCase {
                input: "[1, 2, 3][0 + 2]",
                expected: Object::Integer(3),
            },
            VmTestCase {
                input: "[1, 2, 3][0]",
                expected: Object::Integer(1),
            },
            VmTestCase {
                input: "[[1, 1, 1]][0][0]",
                expected: Object::Integer(1),
            },
            VmTestCase {
                input: "[][0]",
                expected: Object::Null,
            },
            VmTestCase {
                input: "[1, 2, 3][99]",
                expected: Object::Null,
            },
            VmTestCase {
                input: "[1][-1]",
                expected: Object::Null,
            },
            VmTestCase {
                input: "{1: 1, 2: 2}[1]",
                expected: Object::Integer(1),
            },
            VmTestCase {
                input: "{1: 1, 2: 2}[2]",
                expected: Object::Integer(2),
            },
            VmTestCase {
                input: "{1: 1}[0]",
                expected: Object::Null,
            },
            VmTestCase {
                input: "{}[0]",
                expected: Object::Null,
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn test_while_loops() {
        let tests = vec![
            VmTestCase {
                input: "let x = 0; while (x < 5) { let x = x + 1; }; x",
                expected: Object::Integer(5),
            },
            VmTestCase {
                input: "let a = []; let i = 0; while (i < 3) { let a = push(a, i); let i = i + 1; }; a",
                expected: Object::Array(vec![
                    Rc::new(Object::Integer(0)),
                    Rc::new(Object::Integer(1)),
                    Rc::new(Object::Integer(2)),
                ]),
            },
            VmTestCase {
                input: "while (false) { 1 }",
                expected: Object::Null,
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn test_lte_gte_modulo() {
        let tests = vec![
            VmTestCase {
                input: "5 <= 5",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "5 <= 4",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "4 <= 5",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "5 >= 5",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "5 >= 6",
                expected: Object::Boolean(false),
            },
            VmTestCase {
                input: "6 >= 5",
                expected: Object::Boolean(true),
            },
            VmTestCase {
                input: "10 % 3",
                expected: Object::Integer(1),
            },
            VmTestCase {
                input: "10 % 5",
                expected: Object::Integer(0),
            },
            VmTestCase {
                input: "7 % 2",
                expected: Object::Integer(1),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn test_string_escape_sequences() {
        let tests = vec![
            VmTestCase {
                input: r#""\thello\nworld""#,
                expected: Object::String("\thello\nworld".to_string()),
            },
            VmTestCase {
                input: r#""hello\\world""#,
                expected: Object::String("hello\\world".to_string()),
            },
            VmTestCase {
                input: r#""say \"hi\"""#,
                expected: Object::String("say \"hi\"".to_string()),
            },
        ];

        run_vm_tests(tests);
    }

    #[test]
    fn test_tail_call_optimization() {
        let tests = vec![
            VmTestCase {
                input: "let countdown = fn(n) { if (n == 0) { return 0; } countdown(n - 1) }; countdown(10)",
                expected: Object::Integer(0),
            },
            VmTestCase {
                input: "let sum = fn(n, acc) { if (n == 0) { return acc; } sum(n - 1, acc + n) }; sum(100, 0)",
                expected: Object::Integer(5050),
            },
        ];

        run_vm_tests(tests);
    }
}
