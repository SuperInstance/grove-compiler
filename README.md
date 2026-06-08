# grove-compiler

[![crates.io](https://img.shields.io/crates/v/grove-compiler.svg)](https://crates.io/crates/grove-compiler)
[![docs.rs](https://docs.rs/grove-compiler/badge.svg)](https://docs.rs/grove-compiler)
[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## The Idea

A compiler pipeline has four natural phases: parse, validate, optimize, emit. These map cleanly to the four seasons. Spring brings new life (tokens become an AST). Summer is growth and checking (type validation, scope resolution). Autumn strips away the unnecessary (dead code, constant folding). Winter crystallizes what remains (bytecode emission).

The emitted bytecode is **balanced ternary** — digits {-1, 0, +1}, written as `Trit::Neg`, `Trit::Zero`, `Trit::Pos`. Why ternary? Because agent instructions in the SuperInstance fleet encode as ternary states: {-1, 0, +1} maps naturally to {inhibit, idle, activate}. Three-state logic is more expressive than binary and more compact than decimal.

## The Language

A small expression language with:
- **Arithmetic**: `+`, `-`, `*`, `/`
- **Variables**: `let x = expr;`
- **Conditionals**: `if expr { ... } else { ... }`
- **Functions**: `fn name(args) { ... }`
- **Comparisons**: `==`, `!=`, `<`, `>`, `<=`, `>=`

Example program:

```
let tempo = 120;
let beat = 1 / tempo * 60;
let pattern = if beat > 0.5 { 1 } else { 0 };
```

## The Four Seasons

### Spring: Lexing + Parsing

```rust
use grove_compiler::spring::{Lexer, Parser};

let tokens = Lexer::new("let x = 2 + 3;").tokenize();
let ast = Parser::new(tokens).parse_program();
// ast = [Stmt::Let("x", Expr::Binary(2, Add, 3))]
```

The lexer handles numbers, identifiers, operators, keywords, and whitespace. The parser is a recursive descent parser producing a typed AST.

### Summer: Type Checking

```rust
use grove_compiler::summer::TypeChecker;

let mut checker = TypeChecker::new();
checker.check_program(&ast)?;
// Validates: variables declared before use, types consistent, no duplicate declarations
```

### Autumn: Optimization

```rust
use grove_compiler::autumn::Optimizer;

let optimized = Optimizer::new().optimize(&ast);
// Applies: constant folding (2+3→5), dead code elimination, strength reduction (x*2→x+x)
```

| Optimization | Example | Savings |
|---|---|---|
| Constant folding | `2 + 3` → `5` | Eliminates runtime arithmetic |
| Dead code elimination | `if false { ... }` → removed | Eliminates unreachable branches |
| Strength reduction | `x * 2` → `x + x` | Addition is cheaper than multiplication |

### Winter: Ternary Bytecode Emission

```rust
use grove_compiler::winter::Emitter;

let bytecode = Emitter::new().emit(&optimized);
for instr in &bytecode.instructions {
    println!("{:?}", instr);
}
// Each instruction has an opcode (Trit) and operand (Vec<Trit>)
```

**Balanced ternary encoding**: Numbers are encoded in balanced ternary (signed ternary). The value 5 encodes as [Pos, Neg, Neg] because 1·9 + (-1)·3 + (-1)·1 = 5... wait, actually 1·9 + (-1)·3 + (-1)·1 = 5. The crate handles encoding/decoding automatically.

## Full Pipeline

```rust
use grove_compiler::{spring, summer, autumn, winter};

let source = "let x = 2 + 3; if x > 4 { x } else { 0 }";

// Spring: parse
let ast = spring::Parser::new(spring::Lexer::new(source).tokenize()).parse_program()?;

// Summer: typecheck
summer::TypeChecker::new().check_program(&ast)?;

// Autumn: optimize (2+3 folds to 5, dead branch eliminated)
let optimized = autumn::Optimizer::new().optimize(&ast);

// Winter: emit ternary bytecode
let bytecode = winter::Emitter::new().emit(&optimized);
println!("{} instructions emitted", bytecode.instructions.len());
```

## Module Map

| Module | What it does |
|---|---|
| `token` | `Token` enum — all lexical tokens |
| `ast` | `Expr`, `Stmt`, `Program`, `Trit`, `TernaryBytecode` — AST + ternary types |
| `spring` | `Lexer`, `Parser` — tokenization and recursive descent parsing |
| `summer` | `TypeChecker` — scope validation, type consistency |
| `autumn` | `Optimizer` — constant folding, dead code elimination, strength reduction |
| `winter` | `Emitter` — AST → balanced ternary bytecode |
| `error` | `GroveError` with seasonal context |

## Design Decisions

- **Why balanced ternary?** Binary loses the zero state (you get -1 and +1 but not "idle"). Ternary encodes the natural three-state logic of agent actions: inhibit/idle/activate.
- **Why season metaphor?** Because the pipeline stages have genuine seasonal character. Spring creates, summer validates, autumn strips, winter crystallizes. It's also memorable — "check autumn for optimizations" sticks better than "check phase 3."
- **Why not LLVM?** This compiler targets a custom ternary VM for agent instructions, not general-purpose hardware. LLVM doesn't have a ternary backend.

## Links

- [Documentation](https://docs.rs/grove-compiler)
- [Repository](https://github.com/SuperInstance/grove-compiler)
- [crates.io](https://crates.io/crates/grove-compiler)
- See also: [fibration-timing](https://crates.io/crates/fibration-timing) for scheduling agent bytecode execution

## License

MIT
