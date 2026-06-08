use std::collections::HashSet;

use crate::ast::{BinOp, Expr, Program, Stmt};
use crate::error::GroveError;

/// Summer: type checker and scope validator.
///
/// Validates that all referenced variables are declared and that
/// expression types are consistent.
///
/// Symbol table for tracking declared variables.
#[derive(Debug, Clone)]
pub struct Scope {
    vars: HashSet<String>,
}

impl Scope {
    fn new() -> Self {
        Self {
            vars: HashSet::new(),
        }
    }

    fn declare(&mut self, name: &str) {
        self.vars.insert(name.to_string());
    }

    fn is_declared(&self, name: &str) -> bool {
        self.vars.contains(name)
    }
}

/// Type check a full program.
pub fn summer(program: &Program) -> Result<(), GroveError> {
    let mut scope = Scope::new();
    for stmt in &program.stmts {
        check_stmt(stmt, &mut scope)?;
    }
    Ok(())
}

fn check_stmt(stmt: &Stmt, scope: &mut Scope) -> Result<(), GroveError> {
    match stmt {
        Stmt::Let(name, expr) => {
            check_expr(expr, scope)?;
            scope.declare(name);
            Ok(())
        }
        Stmt::Assign(name, expr) => {
            if !scope.is_declared(name) {
                return Err(GroveError::TypeCheck {
                    message: format!("undefined variable: {name}"),
                });
            }
            check_expr(expr, scope)
        }
        Stmt::If(cond, then_branch, else_branch) => {
            check_expr(cond, scope)?;
            let mut inner = scope.clone();
            for s in then_branch {
                check_stmt(s, &mut inner)?;
            }
            for s in else_branch {
                check_stmt(s, &mut inner)?;
            }
            Ok(())
        }
        Stmt::Return(expr) => check_expr(expr, scope),
        Stmt::Expr(expr) => check_expr(expr, scope),
    }
}

fn check_expr(expr: &Expr, scope: &Scope) -> Result<(), GroveError> {
    match expr {
        Expr::Literal(_) => Ok(()),
        Expr::Var(name) => {
            if scope.is_declared(name) {
                Ok(())
            } else {
                Err(GroveError::TypeCheck {
                    message: format!("undefined variable: {name}"),
                })
            }
        }
        Expr::Binary(left, _op, right) => {
            check_expr(left, scope)?;
            check_expr(right, scope)
        }
        Expr::Unary(_, e) => check_expr(e, scope),
        Expr::If(cond, then_e, else_e) => {
            check_expr(cond, scope)?;
            check_expr(then_e, scope)?;
            check_expr(else_e, scope)
        }
        Expr::Call(_name, args) => {
            for arg in args {
                check_expr(arg, scope)?;
            }
            Ok(())
        }
    }
}

/// Constant-evaluate an expression if possible.
pub fn const_eval(expr: &Expr) -> Option<f64> {
    match expr {
        Expr::Literal(n) => Some(*n),
        Expr::Binary(l, op, r) => {
            let lv = const_eval(l)?;
            let rv = const_eval(r)?;
            Some(match op {
                BinOp::Add => lv + rv,
                BinOp::Sub => lv - rv,
                BinOp::Mul => lv * rv,
                BinOp::Div => {
                    if rv == 0.0 {
                        return None;
                    }
                    lv / rv
                }
                BinOp::Eq => f64::from((lv == rv) as u8),
                BinOp::Neq => f64::from((lv != rv) as u8),
                BinOp::Lt => f64::from((lv < rv) as u8),
                BinOp::Gt => f64::from((lv > rv) as u8),
                BinOp::Le => f64::from((lv <= rv) as u8),
                BinOp::Ge => f64::from((lv >= rv) as u8),
            })
        }
        Expr::Unary(crate::ast::UnOp::Neg, e) => Some(-const_eval(e)?),
        _ => None,
    }
}
