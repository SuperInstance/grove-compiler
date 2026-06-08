use serde::{Deserialize, Serialize};

/// A single lexical token produced by the Spring lexer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
    Let,
    If,
    Else,
    Fn,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,
    Assign,
    Return,
    Eof,
}

impl Token {
    /// Returns a human-readable name for the token variant.
    pub fn name(&self) -> &'static str {
        match self {
            Token::Number(_) => "number",
            Token::Ident(_) => "identifier",
            Token::Plus => "+",
            Token::Minus => "-",
            Token::Star => "*",
            Token::Slash => "/",
            Token::Eq => "==",
            Token::Neq => "!=",
            Token::Lt => "<",
            Token::Gt => ">",
            Token::Le => "<=",
            Token::Ge => ">=",
            Token::Let => "let",
            Token::If => "if",
            Token::Else => "else",
            Token::Fn => "fn",
            Token::LParen => "(",
            Token::RParen => ")",
            Token::LBrace => "{",
            Token::RBrace => "}",
            Token::Semicolon => ";",
            Token::Comma => ",",
            Token::Assign => "=",
            Token::Return => "return",
            Token::Eof => "eof",
        }
    }
}
