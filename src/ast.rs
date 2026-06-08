use serde::{Deserialize, Serialize};

use crate::token::Token;

/// Binary operators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
}

impl BinOp {
    /// Convert from token to binary operator, if applicable.
    pub fn from_token(tok: &Token) -> Option<Self> {
        match tok {
            Token::Plus => Some(BinOp::Add),
            Token::Minus => Some(BinOp::Sub),
            Token::Star => Some(BinOp::Mul),
            Token::Slash => Some(BinOp::Div),
            Token::Eq => Some(BinOp::Eq),
            Token::Neq => Some(BinOp::Neq),
            Token::Lt => Some(BinOp::Lt),
            Token::Gt => Some(BinOp::Gt),
            Token::Le => Some(BinOp::Le),
            Token::Ge => Some(BinOp::Ge),
            _ => None,
        }
    }
}

/// Unary operators.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnOp {
    Neg,
}

/// Expression nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expr {
    Literal(f64),
    Var(String),
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

/// Statement nodes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Stmt {
    Let(String, Expr),
    Assign(String, Expr),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    Return(Expr),
    Expr(Expr),
}

/// A full program: a list of statements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

/// Balanced ternary trit: {-1, 0, +1}.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trit {
    Neg,
    Zero,
    Pos,
}

impl Trit {
    /// Convert to integer value.
    pub fn value(self) -> i8 {
        match self {
            Trit::Neg => -1,
            Trit::Zero => 0,
            Trit::Pos => 1,
        }
    }

    /// Create from integer value.
    pub fn from_value(v: i8) -> Self {
        match v {
            -1 => Trit::Neg,
            0 => Trit::Zero,
            1 => Trit::Pos,
            _ => panic!("trit value must be -1, 0, or 1, got {v}"),
        }
    }
}

/// A single ternary instruction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TernaryInstruction {
    pub opcode: Trit,
    pub operand: Vec<Trit>,
}

/// Ternary bytecode: a sequence of ternary instructions plus a constant pool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TernaryBytecode {
    pub instructions: Vec<TernaryInstruction>,
    pub constants: Vec<f64>,
}

impl Default for TernaryBytecode {
    fn default() -> Self {
        Self::new()
    }
}

impl TernaryBytecode {
    /// Create an empty bytecode object.
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
        }
    }
}

/// Balanced ternary number encoding utilities.
pub mod ternary {
    use crate::ast::Trit;

    /// Encode an integer into balanced ternary trits (most-significant first).
    pub fn encode_int(mut n: i64) -> Vec<Trit> {
        if n == 0 {
            return vec![Trit::Zero];
        }
        let mut trits = Vec::new();
        while n != 0 {
            let rem = n.rem_euclid(3);
            match rem {
                0 => trits.push(Trit::Zero),
                1 => trits.push(Trit::Pos),
                2 => trits.push(Trit::Neg),
                _ => unreachable!(),
            }
            n = (n - if rem == 2 { -1 } else { rem }) / 3;
        }
        trits.reverse();
        trits
    }

    /// Decode balanced ternary trits back to an integer.
    pub fn decode_int(trits: &[Trit]) -> i64 {
        let mut val: i64 = 0;
        for &t in trits {
            val = val * 3 + t.value() as i64;
        }
        val
    }
}
