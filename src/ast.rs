//! Abstract Syntax Tree types for the Grove compiler.
//!
//! The AST is the grove — a living forest of expression trees that grow during
//! spring (parsing), are tended through summer (type checking), pruned in
//! autumn (optimization), and harvested in winter (code generation).

use serde::{Deserialize, Serialize};

/// Binary operator kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(Copy)]
pub enum BinOpKind {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
}

/// Unary operator kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[derive(Copy)]
pub enum UnaryKind {
    Neg,
    Not,
}

/// A expression in the grove AST.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    /// Numeric literal.
    Lit(f64),
    /// Variable reference.
    Var(String),
    /// Binary operation: left `op` right.
    BinOp(Box<Expr>, BinOpKind, Box<Expr>),
    /// Unary operation: `op` expr.
    Unary(UnaryKind, Box<Expr>),
    /// Conditional: `if condition then else`.
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    /// Let binding: `let name = value in body`.
    Let(String, Box<Expr>, Box<Expr>),
    /// Ternary: `condition ? then : else`.
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
}

/// Type classification for expressions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Type {
    /// Integer / numeric type.
    Int,
    /// Boolean type.
    Bool,
    /// Ternary type: values in {-1, 0, +1}.
    Ternary,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
            Type::Ternary => write!(f, "Ternary"),
        }
    }
}

/// A type-annotated expression, produced by the summer (type-checking) pass.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TypedExpr {
    pub expr: Expr,
    pub ty: Type,
}

/// A top-level statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    /// Expression statement.
    Expr(Expr),
    /// Let declaration at top level.
    Let(String, Expr),
}

/// A diagnostic message with source span information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub message: String,
    pub start: usize,
    pub end: usize,
}

impl Diagnostic {
    pub fn new(message: impl Into<String>, start: usize, end: usize) -> Self {
        Self {
            message: message.into(),
            start,
            end,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expr_literal() {
        let e = Expr::Lit(42.0);
        assert!(matches!(e, Expr::Lit(v) if v == 42.0));
    }

    #[test]
    fn expr_variable() {
        let e = Expr::Var("x".into());
        assert!(matches!(e, Expr::Var(s) if s == "x"));
    }

    #[test]
    fn expr_binop() {
        let e = Expr::BinOp(Box::new(Expr::Lit(1.0)), BinOpKind::Add, Box::new(Expr::Lit(2.0)));
        if let Expr::BinOp(l, op, r) = e {
            assert_eq!(*l, Expr::Lit(1.0));
            assert_eq!(op, BinOpKind::Add);
            assert_eq!(*r, Expr::Lit(2.0));
        } else {
            panic!("expected BinOp");
        }
    }

    #[test]
    fn type_display() {
        assert_eq!(Type::Int.to_string(), "Int");
        assert_eq!(Type::Bool.to_string(), "Bool");
        assert_eq!(Type::Ternary.to_string(), "Ternary");
    }

    #[test]
    fn typed_expr_construction() {
        let te = TypedExpr {
            expr: Expr::Lit(1.0),
            ty: Type::Ternary,
        };
        assert_eq!(te.ty, Type::Ternary);
    }

    #[test]
    fn diagnostic_new() {
        let d = Diagnostic::new("oops", 0, 5);
        assert_eq!(d.message, "oops");
        assert_eq!(d.start, 0);
        assert_eq!(d.end, 5);
    }
}
