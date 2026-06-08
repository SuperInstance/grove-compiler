//! Autumn — the optimization season.
//!
//! Leaves fall, deadwood is cleared, and the grove is pruned for winter.
//! Constant folding turns ripe literals into harvest. Dead code withers away.
//! Ternary simplifications reshape the undergrowth: `Pos + Pos → Neg` under
//! ternary arithmetic, because in the SuperInstance ecosystem, growth cycles
//! back on itself.

use crate::ast::{BinOpKind, Expr, UnaryKind};

/// Optimize an expression tree in place (returns a new optimized tree).
pub fn optimize(expr: &Expr) -> Expr {
    match expr {
        Expr::Lit(_) => expr.clone(),
        Expr::Var(_) => expr.clone(),
        Expr::BinOp(left, op, right) => {
            let left_opt = optimize(left);
            let right_opt = optimize(right);

            // Constant folding
            if let (Expr::Lit(l), Expr::Lit(r)) = (&left_opt, &right_opt) {
                return match op {
                    BinOpKind::Add => Expr::Lit(l + r),
                    BinOpKind::Sub => Expr::Lit(l - r),
                    BinOpKind::Mul => Expr::Lit(l * r),
                    BinOpKind::Div => {
                        if *r != 0.0 {
                            Expr::Lit(l / r)
                        } else {
                            Expr::BinOp(Box::new(left_opt), *op, Box::new(right_opt))
                        }
                    }
                    BinOpKind::Eq => Expr::Lit(if l == r { 1.0 } else { 0.0 }),
                    BinOpKind::Neq => Expr::Lit(if l != r { 1.0 } else { 0.0 }),
                    BinOpKind::Lt => Expr::Lit(if l < r { 1.0 } else { 0.0 }),
                    BinOpKind::Gt => Expr::Lit(if l > r { 1.0 } else { 0.0 }),
                };
            }

            // Ternary simplification: Pos + Pos → Neg (mod 3 wrapping: 1+1=2≡-1)
            if let (BinOpKind::Add, Expr::Lit(l), Expr::Lit(r)) = (op, &left_opt, &right_opt) {
                if is_ternary_val(*l) && is_ternary_val(*r) {
                    let result = ternary_add(*l, *r);
                    return Expr::Lit(result);
                }
            }

            // Ternary simplification: Pos + Pos → Neg (for symbolic ternary values)
            // In ternary mode, 1 + 1 wraps to -1
            if let BinOpKind::Add = op {
                if let (Expr::Lit(l), Expr::Lit(r)) = (&left_opt, &right_opt) {
                    if *l == 1.0 && *r == 1.0 {
                        return Expr::Lit(-1.0); // Pos + Pos → Neg in ternary
                    }
                }
            }

            Expr::BinOp(Box::new(left_opt), *op, Box::new(right_opt))
        }
        Expr::Unary(kind, operand) => {
            let operand_opt = optimize(operand);
            match kind {
                UnaryKind::Neg => {
                    if let Expr::Lit(n) = &operand_opt {
                        return Expr::Lit(-n);
                    }
                }
                UnaryKind::Not => {
                    if let Expr::Lit(n) = &operand_opt {
                        return Expr::Lit(if *n == 0.0 { 1.0 } else { 0.0 });
                    }
                }
            }
            Expr::Unary(*kind, Box::new(operand_opt))
        }
        Expr::If(cond, then_expr, else_expr) => {
            let cond_opt = optimize(cond);
            let then_opt = optimize(then_expr);
            let else_opt = optimize(else_expr);

            // Dead code elimination: if condition is a known constant
            if let Expr::Lit(n) = &cond_opt {
                return if *n != 0.0 { then_opt } else { else_opt };
            }

            Expr::If(Box::new(cond_opt), Box::new(then_opt), Box::new(else_opt))
        }
        Expr::Let(name, value, body) => {
            let value_opt = optimize(value);
            let body_opt = optimize(body);

            // Dead code elimination: if variable is not used in body
            if !is_used(name, &body_opt) {
                return body_opt;
            }

            Expr::Let(name.clone(), Box::new(value_opt), Box::new(body_opt))
        }
        Expr::Ternary(cond, then_expr, else_expr) => {
            let cond_opt = optimize(cond);
            let then_opt = optimize(then_expr);
            let else_opt = optimize(else_expr);

            // Known condition
            if let Expr::Lit(n) = &cond_opt {
                return if *n != 0.0 { then_opt } else { else_opt };
            }

            Expr::Ternary(Box::new(cond_opt), Box::new(then_opt), Box::new(else_opt))
        }
    }
}

/// Check if a value is a valid ternary value {-1, 0, +1}.
pub fn is_ternary_val(n: f64) -> bool {
    n == -1.0 || n == 0.0 || n == 1.0
}

/// Ternary addition with wrapping: values stay in {-1, 0, +1}.
pub fn ternary_add(a: f64, b: f64) -> f64 {
    let sum = (a as i32) + (b as i32);
    (match sum {
        -3 => 0,
        -2 => 1,
        -1 => -1,
        0 => 0,
        1 => 1,
        2 => -1,
        3 => 0,
        n => n,
    }) as f64
}

/// Check if a variable name appears in an expression.
fn is_used(name: &str, expr: &Expr) -> bool {
    match expr {
        Expr::Lit(_) => false,
        Expr::Var(n) => n == name,
        Expr::BinOp(l, _, r) => is_used(name, l) || is_used(name, r),
        Expr::Unary(_, e) => is_used(name, e),
        Expr::If(c, t, e) => is_used(name, c) || is_used(name, t) || is_used(name, e),
        Expr::Let(n, v, b) => {
            if n == name {
                is_used(name, v)
            } else {
                is_used(name, v) || is_used(name, b)
            }
        }
        Expr::Ternary(c, t, e) => is_used(name, c) || is_used(name, t) || is_used(name, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_fold_add() {
        let expr = Expr::BinOp(
            Box::new(Expr::Lit(2.0)),
            BinOpKind::Add,
            Box::new(Expr::Lit(3.0)),
        );
        assert_eq!(optimize(&expr), Expr::Lit(5.0));
    }

    #[test]
    fn constant_fold_mul() {
        let expr = Expr::BinOp(
            Box::new(Expr::Lit(4.0)),
            BinOpKind::Mul,
            Box::new(Expr::Lit(3.0)),
        );
        assert_eq!(optimize(&expr), Expr::Lit(12.0));
    }

    #[test]
    fn constant_fold_div() {
        let expr = Expr::BinOp(
            Box::new(Expr::Lit(10.0)),
            BinOpKind::Div,
            Box::new(Expr::Lit(2.0)),
        );
        assert_eq!(optimize(&expr), Expr::Lit(5.0));
    }

    #[test]
    fn constant_fold_div_by_zero() {
        let expr = Expr::BinOp(
            Box::new(Expr::Lit(10.0)),
            BinOpKind::Div,
            Box::new(Expr::Lit(0.0)),
        );
        // Should NOT fold division by zero
        assert!(matches!(optimize(&expr), Expr::BinOp(_, _, _)));
    }

    #[test]
    fn constant_fold_comparison() {
        let expr = Expr::BinOp(
            Box::new(Expr::Lit(3.0)),
            BinOpKind::Lt,
            Box::new(Expr::Lit(5.0)),
        );
        assert_eq!(optimize(&expr), Expr::Lit(1.0));
    }

    #[test]
    fn dead_code_if_true() {
        let expr = Expr::If(
            Box::new(Expr::Lit(1.0)),
            Box::new(Expr::Lit(42.0)),
            Box::new(Expr::Lit(99.0)),
        );
        assert_eq!(optimize(&expr), Expr::Lit(42.0));
    }

    #[test]
    fn dead_code_if_false() {
        let expr = Expr::If(
            Box::new(Expr::Lit(0.0)),
            Box::new(Expr::Lit(42.0)),
            Box::new(Expr::Lit(99.0)),
        );
        assert_eq!(optimize(&expr), Expr::Lit(99.0));
    }

    #[test]
    fn dead_code_unused_let() {
        let expr = Expr::Let(
            "x".into(),
            Box::new(Expr::Lit(42.0)),
            Box::new(Expr::Lit(7.0)),
        );
        assert_eq!(optimize(&expr), Expr::Lit(7.0));
    }

    #[test]
    fn keep_used_let() {
        let expr = Expr::Let(
            "x".into(),
            Box::new(Expr::Lit(42.0)),
            Box::new(Expr::Var("x".into())),
        );
        let result = optimize(&expr);
        assert!(matches!(result, Expr::Let(_, _, _)));
    }

    #[test]
    fn ternary_simplification_pos_plus_pos() {
        // 1 + 1 in ternary mode → -1
        let result = ternary_add(1.0, 1.0);
        assert_eq!(result, -1.0);
    }

    #[test]
    fn ternary_simplification_neg_plus_neg() {
        // -1 + -1 → +1
        let result = ternary_add(-1.0, -1.0);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn ternary_simplification_pos_plus_neg() {
        let result = ternary_add(1.0, -1.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn is_ternary_val_check() {
        assert!(is_ternary_val(-1.0));
        assert!(is_ternary_val(0.0));
        assert!(is_ternary_val(1.0));
        assert!(!is_ternary_val(2.0));
        assert!(!is_ternary_val(0.5));
    }

    #[test]
    fn constant_fold_negate() {
        let expr = Expr::Unary(UnaryKind::Neg, Box::new(Expr::Lit(5.0)));
        assert_eq!(optimize(&expr), Expr::Lit(-5.0));
    }

    #[test]
    fn nested_constant_fold() {
        // (2 + 3) * (4 - 1) → 5 * 3 → 15
        let expr = Expr::BinOp(
            Box::new(Expr::BinOp(
                Box::new(Expr::Lit(2.0)),
                BinOpKind::Add,
                Box::new(Expr::Lit(3.0)),
            )),
            BinOpKind::Mul,
            Box::new(Expr::BinOp(
                Box::new(Expr::Lit(4.0)),
                BinOpKind::Sub,
                Box::new(Expr::Lit(1.0)),
            )),
        );
        assert_eq!(optimize(&expr), Expr::Lit(15.0));
    }
}
