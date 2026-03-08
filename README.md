# Monkey Compiler in Rust

A complete implementation of the Monkey programming language compiler and virtual machine, written in Rust. This project is based on Thorsten Ball's excellent book "Writing a Compiler in Go" but reimagined in Rust with modern language features and improved performance.

## What is Monkey?

Monkey is a simple but powerful programming language designed for learning compiler construction. It features:

- **Lexical Analysis**: Token-based parsing with support for identifiers, literals, and operators
- **Parsing**: Recursive descent parser with Pratt parsing for expressions
- **Compilation**: Bytecode compiler that generates instructions for a stack-based VM
- **Virtual Machine**: Stack-based VM with support for functions, closures, and builtins
- **Memory Management**: Automatic memory management with Rust's ownership system

## Features

### Language Features
- **Primitive Types**: Integers, Booleans, Strings, Arrays, and Hash Maps
- **Control Flow**: If/else expressions and `while` loops
- **Functions**: First-class functions with closures and lexical scoping
- **Built-in Functions**: `len()`, `first()`, `last()`, `rest()`, `push()`, `puts()`, and `print()`
- **Operators**: Arithmetic (`+`, `-`, `*`, `/`, `%`), comparison (`==`, `!=`, `<`, `>`, `<=`, `>=`), logical (`!`)

### Compiler Features
- **Multi-pass Compilation**: Lexical analysis → Parsing → Compilation → VM execution
- **Symbol Resolution**: Global and local variable scoping with closure support
- **Bytecode Generation**: Optimized instruction set with constant folding
- **Error Handling**: Comprehensive error reporting and recovery

## Quick Start

### Prerequisites
- Rust 1.70+ and Cargo
- macOS, Linux, or Windows

### Installation

1. **Clone the repository**:
   ```bash
   git clone <repository-url>
   cd compiler
   ```

2. **Build the project**:
   ```bash
   cargo build
   ```

3. **Run the interactive REPL**:
   ```bash
   cargo run --bin monkey-compiler
   ```

### Usage Examples

#### Basic Arithmetic
```monkey
let x = 5 + 3 * 2;
puts(x);  // Output: 11
```

#### Functions and Closures
```monkey
let add = fn(x, y) { x + y };
let result = add(10, 5);
puts(result);  // Output: 15

let make_adder = fn(x) {
    fn(y) { x + y }
};
let add_two = make_adder(2);
puts(add_two(5));  // Output: 7
```

#### Arrays and Hash Maps
```monkey
let arr = [1, 2, 3, 4];
puts(len(arr));  // Output: 4
puts(first(arr));  // Output: 1

let person = {"name": "Alice", "age": 30};
puts(person["name"]);  // Output: Alice
```

#### Control Flow
```monkey
let x = 10;
if (x > 5) {
    puts("x is greater than 5");
} else {
    puts("x is less than or equal to 5");
}
```

## Project Structure

```
compiler/
├── lexer/         # Lexical analysis and tokenization
├── parser/        # AST construction and parsing
├── object/        # Runtime objects and builtins
├── interpreter/   # Tree-walking evaluator
└── compiler/      # Bytecode compiler, opcodes, VM, and REPL
```

### Key Components

- **`lexer/`**: Tokenizes source code into a stream of tokens
- **`parser/`**: Builds an Abstract Syntax Tree (AST) from tokens
- **`compiler/`**: Compiles AST into bytecode instructions and executes them on a stack-based VM
- **`object/`**: Defines runtime objects and built-in functions
- **`interpreter/`**: Provides a tree-walking interpreter for comparison and testing

## Testing

Run the test suite:

```bash
cargo test
```

The project includes comprehensive tests for:
- Lexical analysis
- Parsing
- Compilation
- Virtual machine execution
- Built-in functions

## Development

### Building from Source

```bash
# Build all crates
cargo build

# Build with optimizations
cargo build --release

# Run tests
cargo test

# Check for warnings
cargo check
```

### Adding New Features

1. **Language Features**: Extend the lexer, parser, and compiler
2. **Built-in Functions**: Add to `object/builtins.rs`
3. **Optimizations**: Improve the bytecode compiler or VM
4. **Error Handling**: Enhance error reporting and recovery

### Code Style

This project follows Rust conventions:
- Use `cargo fmt` for code formatting
- Use `cargo clippy` for linting
- Follow Rust naming conventions
- Document public APIs with doc comments

## Learning Resources

- **Original Book**: [Writing a Compiler in Go](https://compilerbook.com/) by Thorsten Ball
- **Rust Documentation**: [The Rust Programming Language](https://doc.rust-lang.org/book/)
- **Compiler Theory**: [Crafting Interpreters](https://craftinginterpreters.com/) by Robert Nystrom

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development Setup

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Thorsten Ball** for the original Go implementation and excellent book
- **The Rust Community** for the amazing language and ecosystem
- **All Contributors** who have helped improve this project

---

**Happy coding!**
