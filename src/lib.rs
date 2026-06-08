//! # grove-compiler
//!
//! Season-based compiler pipeline with ternary bytecode emission.
//!
//! The four seasons of compilation:
//! - **Spring** — Lexing and parsing (source → tokens → AST)
//! - **Summer** — Type checking and scope validation
//! - **Autumn** — Optimization (constant folding, dead code elimination, strength reduction)
//! - **Winter** — Emission of balanced ternary bytecode {-1, 0, +1}
//!
//! ## Example
//!
//! ```rust
//! use grove_compiler::spring;
//!
//! let program = spring("let x = 2 + 3;").unwrap();
//! assert_eq!(program.stmts.len(), 1);
//! ```

pub mod ast;
pub mod autumn;
pub mod error;
pub mod spring;
pub mod summer;
pub mod token;
pub mod winter;

pub use ast::{BinOp, Expr, Program, Stmt, TernaryBytecode, TernaryInstruction, Trit};
pub use autumn::autumn;
pub use error::GroveError;
pub use spring::spring;
pub use summer::summer;
pub use token::Token;
pub use winter::winter;
