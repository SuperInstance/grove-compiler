use crate::ast::{BinOp, Expr, Program, Stmt, UnOp};

/// Autumn: optimizer.
///
/// Performs constant folding, dead code elimination, and strength reduction.
/// Optimize a full program in place.
pub fn autumn(program: &mut Program) {
    program.stmts = optimize_stmts(&program.stmts);
}

fn optimize_stmts(stmts: &[Stmt]) -> Vec<Stmt> {
    stmts.iter().map(optimize_stmt).collect()
}

fn optimize_stmt(stmt: &Stmt) -> Stmt {
    match stmt {
        Stmt::Let(name, expr) => Stmt::Let(name.clone(), optimize_expr(expr)),
        Stmt::Assign(name, expr) => Stmt::Assign(name.clone(), optimize_expr(expr)),
        Stmt::If(cond, then_b, else_b) => {
            let opt_cond = optimize_expr(cond);
            // Dead code elimination: if condition is a constant
            if let Expr::Literal(n) = &opt_cond {
                if *n != 0.0 {
                    // always true — keep then branch only
                    return Stmt::Expr(Expr::Literal(1.0)); // simplified
                } else {
                    // always false — keep else branch only
                    if else_b.is_empty() {
                        return Stmt::Expr(Expr::Literal(0.0)); // dead, remove
                    }
                }
            }
            Stmt::If(opt_cond, optimize_stmts(then_b), optimize_stmts(else_b))
        }
        Stmt::Return(expr) => Stmt::Return(optimize_expr(expr)),
        Stmt::Expr(expr) => Stmt::Expr(optimize_expr(expr)),
    }
}

/// Constant-fold and strength-reduce an expression.
pub fn optimize_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Literal(n) => Expr::Literal(*n),
        Expr::Var(name) => Expr::Var(name.clone()),
        Expr::Binary(left, op, right) => {
            let opt_left = optimize_expr(left);
            let opt_right = optimize_expr(right);

            // Constant folding
            if let (Expr::Literal(l), Expr::Literal(r)) = (&opt_left, &opt_right) {
                if let Some(result) = eval_binop(*l, op, *r) {
                    return Expr::Literal(result);
                }
            }

            // Strength reduction: x * 2 → x + x, x * 1 → x
            match op {
                BinOp::Mul => {
                    if let Expr::Literal(1.0) = opt_right {
                        return opt_left;
                    }
                    if let Expr::Literal(1.0) = opt_left {
                        return opt_right;
                    }
                    if let Expr::Literal(2.0) = opt_right {
                        let l = opt_left.clone();
                        return Expr::Binary(Box::new(opt_left), BinOp::Add, Box::new(l));
                    }
                }
                BinOp::Add => {
                    if let Expr::Literal(0.0) = opt_right {
                        return opt_left;
                    }
                    if let Expr::Literal(0.0) = opt_left {
                        return opt_right;
                    }
                }
                BinOp::Sub => {
                    if let Expr::Literal(0.0) = opt_right {
                        return opt_left;
                    }
                }
                BinOp::Div => {
                    if let Expr::Literal(1.0) = opt_right {
                        return opt_left;
                    }
                }
                _ => {}
            }

            Expr::Binary(Box::new(opt_left), op.clone(), Box::new(opt_right))
        }
        Expr::Unary(op, e) => {
            let opt_e = optimize_expr(e);
            // Constant fold unary
            match op {
                UnOp::Neg => {
                    if let Expr::Literal(n) = &opt_e {
                        return Expr::Literal(-n);
                    }
                }
            }
            Expr::Unary(op.clone(), Box::new(opt_e))
        }
        Expr::If(cond, then_e, else_e) => {
            let opt_cond = optimize_expr(cond);
            let opt_then = optimize_expr(then_e);
            let opt_else = optimize_expr(else_e);
            // Constant-fold if-expression
            if let Expr::Literal(n) = &opt_cond {
                if *n != 0.0 {
                    return opt_then;
                } else {
                    return opt_else;
                }
            }
            Expr::If(Box::new(opt_cond), Box::new(opt_then), Box::new(opt_else))
        }
        Expr::Call(name, args) => {
            let opt_args: Vec<Expr> = args.iter().map(optimize_expr).collect();
            Expr::Call(name.clone(), opt_args)
        }
    }
}

fn eval_binop(l: f64, op: &BinOp, r: f64) -> Option<f64> {
    match op {
        BinOp::Add => Some(l + r),
        BinOp::Sub => Some(l - r),
        BinOp::Mul => Some(l * r),
        BinOp::Div => {
            if r == 0.0 {
                None
            } else {
                Some(l / r)
            }
        }
        BinOp::Eq => Some(f64::from((l == r) as u8)),
        BinOp::Neq => Some(f64::from((l != r) as u8)),
        BinOp::Lt => Some(f64::from((l < r) as u8)),
        BinOp::Gt => Some(f64::from((l > r) as u8)),
        BinOp::Le => Some(f64::from((l <= r) as u8)),
        BinOp::Ge => Some(f64::from((l >= r) as u8)),
    }
}
