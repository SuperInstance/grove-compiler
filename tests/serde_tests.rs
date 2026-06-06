//! Serde round-trip tests to verify all public types serialize/deserialize correctly.

use grove_compiler::*;
use serde_json;

#[test]
fn serde_token_roundtrip() {
    let tokens = vec![
        Token::Num(42.0),
        Token::Ident("foo".into()),
        Token::Plus,
        Token::Minus,
        Token::Star,
        Token::Slash,
        Token::Eq,
        Token::Neq,
        Token::Lt,
        Token::Gt,
        Token::LParen,
        Token::RParen,
        Token::Let,
        Token::If,
        Token::Else,
        Token::Question,
        Token::Colon,
        Token::Assign,
        Token::Eof,
    ];
    for token in &tokens {
        let json = serde_json::to_string(token).unwrap();
        let back: Token = serde_json::from_str(&json).unwrap();
        assert_eq!(*token, back);
    }
}

#[test]
fn serde_expr_roundtrip() {
    let exprs = vec![
        Expr::Lit(42.0),
        Expr::Var("x".into()),
        Expr::BinOp(Box::new(Expr::Lit(1.0)), BinOpKind::Add, Box::new(Expr::Lit(2.0))),
        Expr::Unary(UnaryKind::Neg, Box::new(Expr::Lit(5.0))),
        Expr::If(Box::new(Expr::Lit(1.0)), Box::new(Expr::Lit(2.0)), Box::new(Expr::Lit(3.0))),
        Expr::Let("x".into(), Box::new(Expr::Lit(5.0)), Box::new(Expr::Var("x".into()))),
        Expr::Ternary(Box::new(Expr::Lit(1.0)), Box::new(Expr::Lit(2.0)), Box::new(Expr::Lit(3.0))),
    ];
    for expr in &exprs {
        let json = serde_json::to_string(expr).unwrap();
        let back: Expr = serde_json::from_str(&json).unwrap();
        assert_eq!(*expr, back);
    }
}

#[test]
fn serde_type_roundtrip() {
    for ty in &[Type::Int, Type::Bool, Type::Ternary] {
        let json = serde_json::to_string(ty).unwrap();
        let back: Type = serde_json::from_str(&json).unwrap();
        assert_eq!(*ty, back);
    }
}

#[test]
fn serde_typed_expr_roundtrip() {
    let te = TypedExpr {
        expr: Expr::Lit(1.0),
        ty: Type::Ternary,
    };
    let json = serde_json::to_string(&te).unwrap();
    let back: TypedExpr = serde_json::from_str(&json).unwrap();
    assert_eq!(te, back);
}

#[test]
fn serde_bytecode_roundtrip() {
    let ops = vec![
        Bytecode::Push(42.0),
        Bytecode::Add,
        Bytecode::Sub,
        Bytecode::Mul,
        Bytecode::Div,
        Bytecode::Jump(10),
        Bytecode::JumpIfZero(5),
        Bytecode::Load("x".into()),
        Bytecode::Store("y".into()),
        Bytecode::Halt,
    ];
    for op in &ops {
        let json = serde_json::to_string(op).unwrap();
        let back: Bytecode = serde_json::from_str(&json).unwrap();
        assert_eq!(*op, back);
    }
}

#[test]
fn serde_diagnostic_roundtrip() {
    let d = Diagnostic::new("test error", 5, 10);
    let json = serde_json::to_string(&d).unwrap();
    let back: Diagnostic = serde_json::from_str(&json).unwrap();
    assert_eq!(d, back);
}
