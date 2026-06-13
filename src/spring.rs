//! Spring — the parsing season.
//!
//! Seeds (tokens) germinate into saplings (AST nodes). The parser performs
//! recursive descent through the token stream, growing a grove of expression
//! trees. Errors are captured as spanning diagnostics so the gardener knows
//! exactly where the frost struck.

use crate::ast::{BinOpKind, Diagnostic, Expr, UnaryKind};
use crate::token::{Token, tokenize};

/// A parsed AST plus any diagnostics accumulated during the spring parse.
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub expr: Option<Expr>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Parse a source string into an AST.
pub fn parse(source: &str) -> ParseResult {
    let tokens = tokenize(source);
    let mut parser = Parser::new(&tokens);
    let expr = parser.parse_expr();
    let diagnostics = parser.diagnostics;
    ParseResult {
        expr: Some(expr),
        diagnostics,
    }
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            pos: 0,
            diagnostics: Vec::new(),
        }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> bool {
        if self.peek() == expected {
            self.advance();
            true
        } else {
            self.diagnostics.push(Diagnostic::new(
                format!("expected {:?}, found {:?}", expected, self.peek()),
                self.pos,
                self.pos + 1,
            ));
            false
        }
    }

    fn parse_expr(&mut self) -> Expr {
        self.parse_ternary()
    }

    fn parse_ternary(&mut self) -> Expr {
        let expr = self.parse_comparison();
        if matches!(self.peek(), Token::Question) {
            self.advance();
            let then_expr = self.parse_expr();
            self.expect(&Token::Colon);
            let else_expr = self.parse_expr();
            Expr::Ternary(Box::new(expr), Box::new(then_expr), Box::new(else_expr))
        } else {
            expr
        }
    }

    fn parse_comparison(&mut self) -> Expr {
        let left = self.parse_addition();
        match self.peek() {
            Token::Eq => {
                self.advance();
                let right = self.parse_addition();
                Expr::BinOp(Box::new(left), BinOpKind::Eq, Box::new(right))
            }
            Token::Neq => {
                self.advance();
                let right = self.parse_addition();
                Expr::BinOp(Box::new(left), BinOpKind::Neq, Box::new(right))
            }
            Token::Lt => {
                self.advance();
                let right = self.parse_addition();
                Expr::BinOp(Box::new(left), BinOpKind::Lt, Box::new(right))
            }
            Token::Gt => {
                self.advance();
                let right = self.parse_addition();
                Expr::BinOp(Box::new(left), BinOpKind::Gt, Box::new(right))
            }
            _ => left,
        }
    }

    fn parse_addition(&mut self) -> Expr {
        let mut left = self.parse_multiplication();
        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    let right = self.parse_multiplication();
                    left = Expr::BinOp(Box::new(left), BinOpKind::Add, Box::new(right));
                }
                Token::Minus => {
                    self.advance();
                    let right = self.parse_multiplication();
                    left = Expr::BinOp(Box::new(left), BinOpKind::Sub, Box::new(right));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_multiplication(&mut self) -> Expr {
        let mut left = self.parse_unary();
        loop {
            match self.peek() {
                Token::Star => {
                    self.advance();
                    let right = self.parse_unary();
                    left = Expr::BinOp(Box::new(left), BinOpKind::Mul, Box::new(right));
                }
                Token::Slash => {
                    self.advance();
                    let right = self.parse_unary();
                    left = Expr::BinOp(Box::new(left), BinOpKind::Div, Box::new(right));
                }
                _ => break,
            }
        }
        left
    }

    fn parse_unary(&mut self) -> Expr {
        match self.peek() {
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary();
                Expr::Unary(UnaryKind::Neg, Box::new(expr))
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Expr {
        match self.peek().clone() {
            Token::Num(n) => {
                self.advance();
                Expr::Lit(n)
            }
            Token::Ident(name) => {
                self.advance();
                Expr::Var(name)
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr();
                self.expect(&Token::RParen);
                expr
            }
            Token::If => {
                self.advance();
                let cond = self.parse_expr();
                let then_expr = self.parse_expr();
                self.expect(&Token::Else);
                let else_expr = self.parse_expr();
                Expr::If(Box::new(cond), Box::new(then_expr), Box::new(else_expr))
            }
            Token::Let => {
                self.advance();
                let name = match self.advance() {
                    Token::Ident(n) => n,
                    other => {
                        self.diagnostics.push(Diagnostic::new(
                            format!("expected identifier after 'let', found {:?}", other),
                            self.pos,
                            self.pos + 1,
                        ));
                        String::from("_err")
                    }
                };
                self.expect(&Token::Assign);
                let value = self.parse_expr();
                let body = self.parse_expr();
                Expr::Let(name, Box::new(value), Box::new(body))
            }
            other => {
                self.diagnostics.push(Diagnostic::new(
                    format!("unexpected token: {:?}", other),
                    self.pos,
                    self.pos + 1,
                ));
                self.advance();
                Expr::Lit(0.0)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_number() {
        let result = parse("42");
        assert_eq!(result.expr, Some(Expr::Lit(42.0)));
        assert!(result.diagnostics.is_empty());
    }

    #[test]
    fn parse_addition() {
        let result = parse("1 + 2");
        let expected = Expr::BinOp(
            Box::new(Expr::Lit(1.0)),
            BinOpKind::Add,
            Box::new(Expr::Lit(2.0)),
        );
        assert_eq!(result.expr, Some(expected));
    }

    #[test]
    fn parse_multiplication_precedence() {
        let result = parse("1 + 2 * 3");
        // Should be 1 + (2 * 3)
        let expected = Expr::BinOp(
            Box::new(Expr::Lit(1.0)),
            BinOpKind::Add,
            Box::new(Expr::BinOp(
                Box::new(Expr::Lit(2.0)),
                BinOpKind::Mul,
                Box::new(Expr::Lit(3.0)),
            )),
        );
        assert_eq!(result.expr, Some(expected));
    }

    #[test]
    fn parse_parenthesized() {
        let result = parse("(1 + 2) * 3");
        let expected = Expr::BinOp(
            Box::new(Expr::BinOp(
                Box::new(Expr::Lit(1.0)),
                BinOpKind::Add,
                Box::new(Expr::Lit(2.0)),
            )),
            BinOpKind::Mul,
            Box::new(Expr::Lit(3.0)),
        );
        assert_eq!(result.expr, Some(expected));
    }

    #[test]
    fn parse_unary_negation() {
        let result = parse("-5");
        let expected = Expr::Unary(UnaryKind::Neg, Box::new(Expr::Lit(5.0)));
        assert_eq!(result.expr, Some(expected));
    }

    #[test]
    fn parse_ternary() {
        let result = parse("1 ? 2 : 3");
        let expected = Expr::Ternary(
            Box::new(Expr::Lit(1.0)),
            Box::new(Expr::Lit(2.0)),
            Box::new(Expr::Lit(3.0)),
        );
        assert_eq!(result.expr, Some(expected));
    }

    #[test]
    fn parse_if_else() {
        let result = parse("if 1 2 else 3");
        let expected = Expr::If(
            Box::new(Expr::Lit(1.0)),
            Box::new(Expr::Lit(2.0)),
            Box::new(Expr::Lit(3.0)),
        );
        assert_eq!(result.expr, Some(expected));
    }

    #[test]
    fn parse_let_binding() {
        let result = parse("let x = 5 x");
        let expected = Expr::Let(
            "x".into(),
            Box::new(Expr::Lit(5.0)),
            Box::new(Expr::Var("x".into())),
        );
        assert_eq!(result.expr, Some(expected));
    }

    #[test]
    fn parse_comparison() {
        let result = parse("1 == 2");
        let expected = Expr::BinOp(
            Box::new(Expr::Lit(1.0)),
            BinOpKind::Eq,
            Box::new(Expr::Lit(2.0)),
        );
        assert_eq!(result.expr, Some(expected));
    }

    #[test]
    fn parse_complex_expression() {
        let result = parse("let x = 3 + 4 x * 2");
        assert!(result.diagnostics.is_empty());
        assert!(result.expr.is_some());
    }
}
