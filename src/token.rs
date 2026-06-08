//! Token types for the Grove expression language.
//!
//! The tokenizer breaks raw source text into a stream of [`Token`] values,
//! ready for the spring parser to weave into an AST.

use serde::{Deserialize, Serialize};

/// A lexical token produced by the tokenizer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Token {
    /// Numeric literal (e.g. `42`, `3.14`).
    Num(f64),
    /// Identifier (variable name or keyword payload).
    Ident(String),
    /// `+`
    Plus,
    /// `-`
    Minus,
    /// `*`
    Star,
    /// `/`
    Slash,
    /// `==`
    Eq,
    /// `!=`
    Neq,
    /// `<`
    Lt,
    /// `>`
    Gt,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `let`
    Let,
    /// `if`
    If,
    /// `else`
    Else,
    /// `?`
    Question,
    /// `:`
    Colon,
    /// `=`
    Assign,
    /// End of input.
    Eof,
}

/// Tokenize a source string into a vector of tokens.
///
/// Whitespace is skipped. Unknown characters produce an error message in
/// the result vector (as `Token::Ident` starting with `§ERROR§`).
pub fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '0'..='9' => {
                let mut num_str = String::new();
                while let Some(&d) = chars.peek() {
                    if d.is_ascii_digit() || d == '.' {
                        num_str.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let val: f64 = num_str.parse().unwrap_or(0.0);
                tokens.push(Token::Num(val));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                match ident.as_str() {
                    "let" => tokens.push(Token::Let),
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "true" => tokens.push(Token::Num(1.0)),
                    "false" => tokens.push(Token::Num(0.0)),
                    _ => tokens.push(Token::Ident(ident)),
                }
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            '/' => {
                chars.next();
                tokens.push(Token::Slash);
            }
            '(' => {
                chars.next();
                tokens.push(Token::LParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RParen);
            }
            '?' => {
                chars.next();
                tokens.push(Token::Question);
            }
            ':' => {
                chars.next();
                tokens.push(Token::Colon);
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Eq);
                } else {
                    tokens.push(Token::Assign);
                }
            }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::Neq);
                } else {
                    tokens.push(Token::Ident("§ERROR§unexpected '!'".into()));
                }
            }
            '<' => {
                chars.next();
                tokens.push(Token::Lt);
            }
            '>' => {
                chars.next();
                tokens.push(Token::Gt);
            }
            _ => {
                chars.next();
                tokens.push(Token::Ident(format!("§ERROR§unexpected '{}'", ch)));
            }
        }
    }

    tokens.push(Token::Eof);
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_numbers() {
        let tokens = tokenize("42 3.14");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0], Token::Num(42.0));
        assert_eq!(tokens[1], Token::Num(3.14));
    }

    #[test]
    fn tokenize_identifiers() {
        let tokens = tokenize("foo bar_baz");
        assert_eq!(tokens[0], Token::Ident("foo".into()));
        assert_eq!(tokens[1], Token::Ident("bar_baz".into()));
    }

    #[test]
    fn tokenize_operators() {
        let tokens = tokenize("+ - * / == != < >");
        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Star);
        assert_eq!(tokens[3], Token::Slash);
        assert_eq!(tokens[4], Token::Eq);
        assert_eq!(tokens[5], Token::Neq);
        assert_eq!(tokens[6], Token::Lt);
        assert_eq!(tokens[7], Token::Gt);
    }

    #[test]
    fn tokenize_keywords() {
        let tokens = tokenize("let if else");
        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[1], Token::If);
        assert_eq!(tokens[2], Token::Else);
    }

    #[test]
    fn tokenize_parens_and_ternary() {
        let tokens = tokenize("( ) ? :");
        assert_eq!(tokens[0], Token::LParen);
        assert_eq!(tokens[1], Token::RParen);
        assert_eq!(tokens[2], Token::Question);
        assert_eq!(tokens[3], Token::Colon);
    }

    #[test]
    fn tokenize_assignment() {
        let tokens = tokenize("x = 5");
        assert_eq!(tokens[0], Token::Ident("x".into()));
        assert_eq!(tokens[1], Token::Assign);
        assert_eq!(tokens[2], Token::Num(5.0));
    }

    #[test]
    fn tokenize_eof() {
        let tokens = tokenize("");
        assert_eq!(tokens, vec![Token::Eof]);
    }

    #[test]
    fn tokenize_boolean_literals() {
        let tokens = tokenize("true false");
        assert_eq!(tokens[0], Token::Num(1.0));
        assert_eq!(tokens[1], Token::Num(0.0));
    }
}
