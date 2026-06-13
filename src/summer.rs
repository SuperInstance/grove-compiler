//! Summer — the type-checking season.
//!
//! The canopy thickens. Each tree in the grove is classified by its ecological
//! niche: `Int` for the hardwoods of arithmetic, `Bool` for the deciduous
//! branching logic, and `Ternary` for the rare {-1, 0, +1} undergrowth
//! unique to the SuperInstance ecosystem. The type checker ensures every
//! tree occupies its correct niche.

use crate::ast::{BinOpKind, Expr, Type, TypedExpr, UnaryKind};

/// Result of type-checking an expression.
#[derive(Debug, Clone)]
pub struct TypeCheckResult {
    pub typed: Option<TypedExpr>,
    pub errors: Vec<String>,
}

/// Type-check an expression, producing a typed AST or errors.
pub fn typecheck(expr: &Expr) -> TypeCheckResult {
    let mut checker = TypeChecker::new();
    let typed = checker.check(expr);
    TypeCheckResult {
        typed: Some(typed),
        errors: checker.errors,
    }
}

struct TypeChecker {
    errors: Vec<String>,
    /// Variable type environment.
    env: Vec<(String, Type)>,
}

impl TypeChecker {
    fn new() -> Self {
        Self {
            errors: Vec::new(),
            env: Vec::new(),
        }
    }

    fn check(&mut self, expr: &Expr) -> TypedExpr {
        match expr {
            Expr::Lit(n) => {
                // Check for exact ternary values {-1, 0, +1}
                let ty = if *n == -1.0 || *n == 0.0 || *n == 1.0 {
                    Type::Ternary
                } else {
                    Type::Int
                };
                TypedExpr {
                    expr: expr.clone(),
                    ty,
                }
            }
            Expr::Var(name) => {
                let ty = self.env.iter().rev()
                    .find(|(n, _)| n == name)
                    .map(|(_, t)| t.clone())
                    .unwrap_or_else(|| {
                        self.errors.push(format!("undefined variable: {}", name));
                        Type::Int
                    });
                TypedExpr {
                    expr: expr.clone(),
                    ty,
                }
            }
            Expr::BinOp(left, op, right) => {
                let left_typed = self.check(left);
                let right_typed = self.check(right);
                let ty = match op {
                    BinOpKind::Add | BinOpKind::Sub | BinOpKind::Mul | BinOpKind::Div => {
                        if left_typed.ty == Type::Ternary && right_typed.ty == Type::Ternary {
                            Type::Ternary
                        } else {
                            Type::Int
                        }
                    }
                    BinOpKind::Eq | BinOpKind::Neq | BinOpKind::Lt | BinOpKind::Gt => Type::Bool,
                };
                TypedExpr {
                    expr: expr.clone(),
                    ty,
                }
            }
            Expr::Unary(kind, operand) => {
                let operand_typed = self.check(operand);
                let ty = match kind {
                    UnaryKind::Neg => operand_typed.ty.clone(),
                    UnaryKind::Not => Type::Bool,
                };
                TypedExpr {
                    expr: expr.clone(),
                    ty,
                }
            }
            Expr::If(cond, then_expr, else_expr) => {
                let cond_typed = self.check(cond);
                let then_typed = self.check(then_expr);
                let else_typed = self.check(else_expr);
                if cond_typed.ty != Type::Bool && cond_typed.ty != Type::Ternary {
                    self.errors.push(format!(
                        "if condition must be Bool or Ternary, got {}",
                        cond_typed.ty
                    ));
                }
                let ty = if then_typed.ty == else_typed.ty {
                    then_typed.ty.clone()
                } else {
                    self.errors.push(format!(
                        "if branches have mismatched types: {} vs {}",
                        then_typed.ty, else_typed.ty
                    ));
                    then_typed.ty.clone()
                };
                TypedExpr {
                    expr: expr.clone(),
                    ty,
                }
            }
            Expr::Let(name, value, body) => {
                let value_typed = self.check(value);
                self.env.push((name.clone(), value_typed.ty.clone()));
                let body_typed = self.check(body);
                self.env.pop();
                TypedExpr {
                    expr: expr.clone(),
                    ty: body_typed.ty,
                }
            }
            Expr::Ternary(cond, then_expr, else_expr) => {
                let cond_typed = self.check(cond);
                let then_typed = self.check(then_expr);
                let else_typed = self.check(else_expr);
                let ty = if then_typed.ty == else_typed.ty {
                    then_typed.ty.clone()
                } else {
                    Type::Int
                };
                let _ = (cond_typed, cond);
                TypedExpr {
                    expr: expr.clone(),
                    ty,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typecheck_literal_int() {
        let result = typecheck(&Expr::Lit(42.0));
        assert_eq!(result.typed.unwrap().ty, Type::Int);
    }

    #[test]
    fn typecheck_literal_ternary() {
        let result = typecheck(&Expr::Lit(1.0));
        assert_eq!(result.typed.unwrap().ty, Type::Ternary);
    }

    #[test]
    fn typecheck_literal_neg_one() {
        let result = typecheck(&Expr::Lit(-1.0));
        assert_eq!(result.typed.unwrap().ty, Type::Ternary);
    }

    #[test]
    fn typecheck_binop_arithmetic() {
        let expr = Expr::BinOp(Box::new(Expr::Lit(1.0)), BinOpKind::Add, Box::new(Expr::Lit(1.0)));
        let result = typecheck(&expr);
        assert_eq!(result.typed.unwrap().ty, Type::Ternary);
    }

    #[test]
    fn typecheck_binop_comparison() {
        let expr = Expr::BinOp(Box::new(Expr::Lit(5.0)), BinOpKind::Lt, Box::new(Expr::Lit(3.0)));
        let result = typecheck(&expr);
        assert_eq!(result.typed.unwrap().ty, Type::Bool);
    }

    #[test]
    fn typecheck_let_inference() {
        // let x = 42 in x + 1
        let expr = Expr::Let(
            "x".into(),
            Box::new(Expr::Lit(42.0)),
            Box::new(Expr::BinOp(
                Box::new(Expr::Var("x".into())),
                BinOpKind::Add,
                Box::new(Expr::Lit(1.0)),
            )),
        );
        let result = typecheck(&expr);
        assert_eq!(result.typed.unwrap().ty, Type::Int);
    }

    #[test]
    fn typecheck_if_branches_match() {
        let expr = Expr::If(
            Box::new(Expr::BinOp(Box::new(Expr::Lit(1.0)), BinOpKind::Eq, Box::new(Expr::Lit(1.0)))),
            Box::new(Expr::Lit(42.0)),
            Box::new(Expr::Lit(7.0)),
        );
        let result = typecheck(&expr);
        assert_eq!(result.typed.unwrap().ty, Type::Int);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn typecheck_undefined_var() {
        let expr = Expr::Var("undefined_var".into());
        let result = typecheck(&expr);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn typecheck_ternary_expr() {
        let expr = Expr::Ternary(
            Box::new(Expr::Lit(1.0)),
            Box::new(Expr::Lit(1.0)),
            Box::new(Expr::Lit(-1.0)),
        );
        let result = typecheck(&expr);
        assert_eq!(result.typed.unwrap().ty, Type::Ternary);
    }
}
