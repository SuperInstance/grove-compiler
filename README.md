# 🌳 Grove Compiler

*A tree-based compiler for the [SuperInstance](https://github.com/SuperInstance) ecosystem.*

**The forest is the program. The seasons are the compiler passes.**

---

## The Seasons of Compilation

Grove Compiler doesn't just compile — it grows. Every source file is a seed that journeys through four seasons before becoming bytecode:

### 🌱 Spring — Parsing

Seeds germinate. The tokenizer cracks open raw source text into tokens, and the recursive descent parser weaves them into a living grove of AST trees. Saplings emerge: literals, variables, binary operations, if-else branches, let bindings, ternary expressions.

```
source text → tokens → AST (a grove of expression trees)
```

Error recovery spans the frost-damaged regions with diagnostic precision, so the gardener knows exactly where the cold struck.

### ☀️ Summer — Type Checking

The canopy thickens. Each tree in the grove is classified by its **ecological niche**:

- **`Int`** — The hardwoods of arithmetic. Sturdy, numeric, reliable.
- **`Bool`** — The deciduous branching logic. True or false, leaf or bare.
- **`Ternary`** — The rare {-1, 0, +1} undergrowth, unique to the SuperInstance ecosystem.

The type checker walks the grove verifying that every tree occupies its correct niche. Variable bindings are inferred — no explicit annotations needed. Branches must match their types; mismatched canopies are flagged.

### 🍂 Autumn — Optimization

Leaves fall. Deadwood is cleared. The grove is pruned for the lean months ahead.

- **Constant folding** — Ripe expressions (`2 + 3`) are harvested into their values (`5`).
- **Dead code elimination** — Unreachable branches wither away. Unused let bindings decompose.
- **Ternary simplification** — The undergrowth reshapes itself: `Pos + Pos → Neg` under ternary arithmetic, because in the SuperInstance ecosystem, growth cycles back on itself (`1 + 1 ≡ -1 mod 3`).

### ❄️ Winter — Code Generation

The harvest is collected. The grove's fully grown and optimized trees are felled and milled into **bytecode** — stack-machine instructions ready for the virtual machine.

```
AST → Push, Add, Sub, Mul, Div, Jump, JumpIfZero, Load, Store, Halt
```

Each expression becomes a sequence of pushes, operations, jumps, and stores: the lumber of computation. A constant pool is gathered alongside for efficient lookup.

---

## Quick Start

```rust
use grove_compiler::{compile, tokenize, parse, typecheck, optimize, Compiler};

// Full pipeline: all four seasons in one call
let result = grove_compiler::compile("let x = 2 + 3  x * 4");
assert!(result.success);
println!("Bytecode: {:?}", result.bytecode);

// Or walk the seasons yourself:
let tokens = tokenize("1 + 2 * 3");
let parsed = parse("1 + 2 * 3");
if let Some(expr) = parsed.expr {
    let typed = typecheck(&expr);
    let optimized = optimize(&expr);
    let compiler = Compiler::new().compile(&optimized);
    println!("Constants: {:?}", compiler.constants);
    println!("Bytecode: {:?}", compiler.bytecode());
}
```

## Expression Language

The Grove expression language supports:

| Syntax | Description |
|--------|-------------|
| `42`, `3.14` | Numeric literals |
| `x`, `foo_bar` | Identifiers |
| `+`, `-`, `*`, `/` | Arithmetic operators |
| `==`, `!=`, `<`, `>` | Comparison operators |
| `(...)` | Parenthesized grouping |
| `let x = expr body` | Let bindings |
| `if cond then else else` | Conditionals |
| `cond ? then : else` | Ternary expressions |
| `-expr` | Unary negation |
| `true`, `false` | Boolean literals |

### Examples

```rust
use grove_compiler::compile;

// Arithmetic
compile("2 + 3 * 4");         // → Push(14.0), Halt  (constant folded!)

// Variables
compile("let x = 10  x + 5"); // → Push(10), Store(x), Load(x), Push(5), Add, Halt

// Conditionals
compile("if 1 42 else 99");   // → Push(1), JumpIfZero(else), Push(42), Jump(end), Push(99), Halt

// Ternary
compile("1 ? 2 : 3");         // → same bytecode structure as if-else
```

## Architecture

```
src/
├── lib.rs      ← Library root, full pipeline, integration tests
├── token.rs    ← Tokenizer & Token types
├── ast.rs      ← AST, Type, TypedExpr, Diagnostic types
├── spring.rs   ← Parser (spring season): tokens → AST
├── summer.rs   ← Type checker (summer season): AST → TypedAST
├── autumn.rs   ← Optimizer (autumn season): AST → optimized AST
└── winter.rs   ← Code generator (winter season): AST → Bytecode
```

## Core Types

```rust
// Tokens
enum Token { Num(f64), Ident(String), Plus, Minus, Star, Slash, 
             Eq, Neq, Lt, Gt, LParen, RParen, Let, If, Else, 
             Question, Colon, Assign, Eof }

// AST
enum Expr { Lit(f64), Var(String), BinOp(Box<Expr>, BinOpKind, Box<Expr>),
            Unary(UnaryKind, Box<Expr>), If(Box<Expr>, Box<Expr>, Box<Expr>),
            Let(String, Box<Expr>, Box<Expr>), 
            Ternary(Box<Expr>, Box<Expr>, Box<Expr>) }

// Types
enum Type { Int, Bool, Ternary }  // Ternary = {-1, 0, +1}

// Bytecode (stack machine)
enum Bytecode { Push(f64), Add, Sub, Mul, Div, 
                Jump(usize), JumpIfZero(usize),
                Load(String), Store(String), Halt }

// Compiler
struct Compiler { bytecode: Vec<Bytecode>, constants: Vec<f64> }
```

All public types derive `Serialize` and `Deserialize` via [serde](https://serde.rs).

## Ternary Arithmetic

The ternary type system models values in {-1, 0, +1}, with modular addition:

```
 Pos + Pos → Neg   (1 + 1 = -1)
 Neg + Neg → Pos   (-1 + -1 = +1)
 Pos + Neg → Zero  (1 + -1 = 0)
```

This reflects the cyclical nature of the SuperInstance ecosystem, where extreme growth in one direction loops back to its opposite.

## Testing

```bash
cargo test        # 74 tests across all modules
cargo test --doc  # Doc tests including the quick-start example
```

The test suite covers tokenization, parsing, type checking, optimization, code generation, serde round-trips, and end-to-end integration.

## License

MIT

---

*From seed to bytecode, the grove endures.* 🌲
