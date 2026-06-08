//! Winter — the code generation season.
//!
//! The harvest is collected. The grove's fully grown and optimized trees are
//! felled and milled into bytecode — stack-machine instructions ready for
//! the virtual machine. Each expression becomes a sequence of pushes,
//! operations, jumps, and stores: the lumber of computation.

use serde::{Deserialize, Serialize};

use crate::ast::{BinOpKind, Expr, UnaryKind};

/// Bytecode instructions for the stack-based virtual machine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Bytecode {
    /// Push a numeric constant onto the stack.
    Push(f64),
    /// Add top two stack values.
    Add,
    /// Subtract top from second.
    Sub,
    /// Multiply top two stack values.
    Mul,
    /// Divide second by top.
    Div,
    /// Unconditional jump to address.
    Jump(usize),
    /// Jump if top of stack is zero.
    JumpIfZero(usize),
    /// Load variable by name.
    Load(String),
    /// Store top of stack into variable.
    Store(String),
    /// Halt execution.
    Halt,
}

/// The winter compiler: transforms an AST into bytecode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compiler {
    /// Emitted bytecode instructions.
    pub bytecode: Vec<Bytecode>,
    /// Collected numeric constants (for constant pool).
    pub constants: Vec<f64>,
}

impl Compiler {
    /// Create a new compiler with empty bytecode and constants.
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            constants: Vec::new(),
        }
    }

    /// Compile an expression into bytecode, returning the compiler.
    pub fn compile(mut self, expr: &Expr) -> Self {
        self.emit_expr(expr);
        self.emit(Bytecode::Halt);
        self
    }

    /// Get the compiled bytecode slice.
    pub fn bytecode(&self) -> &[Bytecode] {
        &self.bytecode
    }

    fn emit(&mut self, op: Bytecode) {
        self.bytecode.push(op);
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Lit(n) => {
                self.constants.push(*n);
                self.emit(Bytecode::Push(*n));
            }
            Expr::Var(name) => {
                self.emit(Bytecode::Load(name.clone()));
            }
            Expr::BinOp(left, op, right) => {
                self.emit_expr(left);
                self.emit_expr(right);
                match op {
                    BinOpKind::Add => self.emit(Bytecode::Add),
                    BinOpKind::Sub => self.emit(Bytecode::Sub),
                    BinOpKind::Mul => self.emit(Bytecode::Mul),
                    BinOpKind::Div => self.emit(Bytecode::Div),
                    BinOpKind::Eq => {
                        // Emit comparison: subtract and check zero
                        self.emit(Bytecode::Sub);
                        // For now, we use a simple model: result is 0 (equal) or non-zero
                    }
                    BinOpKind::Neq => {
                        self.emit(Bytecode::Sub);
                    }
                    BinOpKind::Lt | BinOpKind::Gt => {
                        self.emit(Bytecode::Sub);
                    }
                }
            }
            Expr::Unary(kind, operand) => {
                match kind {
                    UnaryKind::Neg => {
                        self.emit(Bytecode::Push(0.0));
                        self.emit_expr(operand);
                        self.emit(Bytecode::Sub);
                    }
                    UnaryKind::Not => {
                        self.emit_expr(operand);
                        // Logical not: push 0 for truthy, 1 for falsy
                        // Simple: check if top is zero → push 1, else push 0
                        // Using JumpIfZero
                        let not_start = self.bytecode.len();
                        self.emit(Bytecode::JumpIfZero(0)); // placeholder
                        self.emit(Bytecode::Push(0.0));
                        self.emit(Bytecode::Jump(0)); // placeholder
                        let push_one_addr = self.bytecode.len();
                        self.emit(Bytecode::Push(1.0));
                        let end_addr = self.bytecode.len();
                        // Patch jumps
                        self.bytecode[not_start] = Bytecode::JumpIfZero(push_one_addr);
                        self.bytecode[not_start + 2] = Bytecode::Jump(end_addr);
                    }
                }
            }
            Expr::If(cond, then_expr, else_expr) => {
                self.emit_expr(cond);
                let jump_if_zero = self.bytecode.len();
                self.emit(Bytecode::JumpIfZero(0)); // placeholder
                self.emit_expr(then_expr);
                let jump_end = self.bytecode.len();
                self.emit(Bytecode::Jump(0)); // placeholder
                let else_start = self.bytecode.len();
                self.emit_expr(else_expr);
                let end = self.bytecode.len();
                // Patch jumps
                self.bytecode[jump_if_zero] = Bytecode::JumpIfZero(else_start);
                self.bytecode[jump_end] = Bytecode::Jump(end);
            }
            Expr::Let(name, value, body) => {
                self.emit_expr(value);
                self.emit(Bytecode::Store(name.clone()));
                self.emit_expr(body);
            }
            Expr::Ternary(cond, then_expr, else_expr) => {
                // Same as if-else in bytecode
                self.emit_expr(cond);
                let jump_if_zero = self.bytecode.len();
                self.emit(Bytecode::JumpIfZero(0));
                self.emit_expr(then_expr);
                let jump_end = self.bytecode.len();
                self.emit(Bytecode::Jump(0));
                let else_start = self.bytecode.len();
                self.emit_expr(else_expr);
                let end = self.bytecode.len();
                self.bytecode[jump_if_zero] = Bytecode::JumpIfZero(else_start);
                self.bytecode[jump_end] = Bytecode::Jump(end);
            }
        }
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_literal() {
        let compiler = Compiler::new().compile(&Expr::Lit(42.0));
        assert_eq!(compiler.bytecode()[0], Bytecode::Push(42.0));
        assert_eq!(compiler.bytecode()[1], Bytecode::Halt);
    }

    #[test]
    fn compile_addition() {
        let expr = Expr::BinOp(
            Box::new(Expr::Lit(1.0)),
            BinOpKind::Add,
            Box::new(Expr::Lit(2.0)),
        );
        let compiler = Compiler::new().compile(&expr);
        assert_eq!(
            compiler.bytecode(),
            &[
                Bytecode::Push(1.0),
                Bytecode::Push(2.0),
                Bytecode::Add,
                Bytecode::Halt,
            ]
        );
    }

    #[test]
    fn compile_subtraction() {
        let expr = Expr::BinOp(
            Box::new(Expr::Lit(5.0)),
            BinOpKind::Sub,
            Box::new(Expr::Lit(3.0)),
        );
        let compiler = Compiler::new().compile(&expr);
        assert_eq!(compiler.bytecode()[2], Bytecode::Sub);
    }

    #[test]
    fn compile_multiplication() {
        let expr = Expr::BinOp(
            Box::new(Expr::Lit(4.0)),
            BinOpKind::Mul,
            Box::new(Expr::Lit(3.0)),
        );
        let compiler = Compiler::new().compile(&expr);
        assert_eq!(compiler.bytecode()[2], Bytecode::Mul);
    }

    #[test]
    fn compile_division() {
        let expr = Expr::BinOp(
            Box::new(Expr::Lit(10.0)),
            BinOpKind::Div,
            Box::new(Expr::Lit(2.0)),
        );
        let compiler = Compiler::new().compile(&expr);
        assert_eq!(compiler.bytecode()[2], Bytecode::Div);
    }

    #[test]
    fn compile_variable_load() {
        let expr = Expr::Var("x".into());
        let compiler = Compiler::new().compile(&expr);
        assert_eq!(compiler.bytecode()[0], Bytecode::Load("x".into()));
    }

    #[test]
    fn compile_let_binding() {
        let expr = Expr::Let(
            "x".into(),
            Box::new(Expr::Lit(5.0)),
            Box::new(Expr::Var("x".into())),
        );
        let compiler = Compiler::new().compile(&expr);
        assert_eq!(compiler.bytecode()[0], Bytecode::Push(5.0));
        assert_eq!(compiler.bytecode()[1], Bytecode::Store("x".into()));
        assert_eq!(compiler.bytecode()[2], Bytecode::Load("x".into()));
    }

    #[test]
    fn compile_if() {
        let expr = Expr::If(
            Box::new(Expr::Lit(1.0)),
            Box::new(Expr::Lit(42.0)),
            Box::new(Expr::Lit(99.0)),
        );
        let compiler = Compiler::new().compile(&expr);
        // Should have: Push(1.0), JumpIfZero, Push(42.0), Jump, Push(99.0), Halt
        assert!(compiler.bytecode().len() >= 5);
        assert_eq!(compiler.bytecode()[0], Bytecode::Push(1.0));
    }

    #[test]
    fn compile_ternary() {
        let expr = Expr::Ternary(
            Box::new(Expr::Lit(1.0)),
            Box::new(Expr::Lit(2.0)),
            Box::new(Expr::Lit(3.0)),
        );
        let compiler = Compiler::new().compile(&expr);
        assert!(compiler.bytecode().len() >= 5);
    }

    #[test]
    fn compile_negation() {
        let expr = Expr::Unary(UnaryKind::Neg, Box::new(Expr::Lit(5.0)));
        let compiler = Compiler::new().compile(&expr);
        assert_eq!(compiler.bytecode()[0], Bytecode::Push(0.0));
        assert_eq!(compiler.bytecode()[1], Bytecode::Push(5.0));
        assert_eq!(compiler.bytecode()[2], Bytecode::Sub);
    }

    #[test]
    fn compile_constants_collected() {
        let expr = Expr::BinOp(
            Box::new(Expr::Lit(1.0)),
            BinOpKind::Add,
            Box::new(Expr::Lit(2.0)),
        );
        let compiler = Compiler::new().compile(&expr);
        assert_eq!(compiler.constants, vec![1.0, 2.0]);
    }

    #[test]
    fn compile_default_compiler() {
        let compiler = Compiler::default();
        assert!(compiler.bytecode.is_empty());
        assert!(compiler.constants.is_empty());
    }
}
