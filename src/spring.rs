use crate::ast::{BinOp, Expr, Program, Stmt, UnOp};
use crate::error::GroveError;
use crate::token::Token;

// Spring: Lexer + Parser.
//
// The lexer tokenizes input text; the parser builds an AST.

// ── Lexer ──────────────────────────────────────────────────────────────────

/// Lexer: converts source text into a token stream.
pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
}

impl Lexer {
    /// Create a new lexer for the given source text.
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' {
                self.advance();
            } else {
                break;
            }
        }
        let s: String = self.chars[start..self.pos].iter().collect();
        Token::Number(s.parse::<f64>().unwrap_or(0.0))
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        self.chars[start..self.pos].iter().collect()
    }

    /// Lex the entire input into a token vector.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, GroveError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => {
                    tokens.push(Token::Eof);
                    return Ok(tokens);
                }
                Some(ch) => {
                    let tok = match ch {
                        '0'..='9' => self.read_number(),
                        'a'..='z' | 'A'..='Z' | '_' => {
                            let word = self.read_ident();
                            match word.as_str() {
                                "let" => Token::Let,
                                "if" => Token::If,
                                "else" => Token::Else,
                                "fn" => Token::Fn,
                                "return" => Token::Return,
                                _ => Token::Ident(word),
                            }
                        }
                        '+' => { self.advance(); Token::Plus }
                        '-' => { self.advance(); Token::Minus }
                        '*' => { self.advance(); Token::Star }
                        '/' => { self.advance(); Token::Slash }
                        '(' => { self.advance(); Token::LParen }
                        ')' => { self.advance(); Token::RParen }
                        '{' => { self.advance(); Token::LBrace }
                        '}' => { self.advance(); Token::RBrace }
                        ';' => { self.advance(); Token::Semicolon }
                        ',' => { self.advance(); Token::Comma }
                        '=' => {
                            self.advance();
                            if self.peek() == Some('=') {
                                self.advance();
                                Token::Eq
                            } else {
                                Token::Assign
                            }
                        }
                        '!' => {
                            self.advance();
                            if self.peek() == Some('=') {
                                self.advance();
                                Token::Neq
                            } else {
                                return Err(GroveError::Lex {
                                    message: "expected '=' after '!'".into(),
                                    pos: self.pos,
                                });
                            }
                        }
                        '<' => {
                            self.advance();
                            if self.peek() == Some('=') {
                                self.advance();
                                Token::Le
                            } else {
                                Token::Lt
                            }
                        }
                        '>' => {
                            self.advance();
                            if self.peek() == Some('=') {
                                self.advance();
                                Token::Ge
                            } else {
                                Token::Gt
                            }
                        }
                        _ => {
                            return Err(GroveError::Lex {
                                message: format!("unexpected character: {ch}"),
                                pos: self.pos,
                            });
                        }
                    };
                    tokens.push(tok);
                }
            }
        }
    }
}

// ── Parser ─────────────────────────────────────────────────────────────────

/// Parser: recursive-descent parser producing an AST.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Create a new parser from a token stream.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let tok = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        tok
    }

    fn expect(&mut self, expected: &Token) -> Result<(), GroveError> {
        let tok = self.advance();
        if tok == *expected {
            Ok(())
        } else {
            Err(GroveError::Parse {
                message: format!("expected {}, got {}", expected.name(), tok.name()),
                pos: self.pos,
            })
        }
    }

    /// Parse a full program.
    pub fn parse_program(&mut self) -> Result<Program, GroveError> {
        let mut stmts = Vec::new();
        while *self.peek() != Token::Eof {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Program { stmts })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, GroveError> {
        match self.peek().clone() {
            Token::Let => self.parse_let(),
            Token::If => self.parse_if_stmt(),
            Token::Return => self.parse_return(),
            Token::Ident(_) => {
                // Could be assignment or expression statement
                let pos = self.pos;
                let name_tok = self.advance();
                if let Token::Ident(name) = name_tok {
                    if *self.peek() == Token::Assign {
                        self.advance();
                        let expr = self.parse_expr()?;
                        self.expect(&Token::Semicolon)?;
                        Ok(Stmt::Assign(name, expr))
                    } else {
                        // rewind and parse as expression
                        self.pos = pos;
                        let expr = self.parse_expr()?;
                        self.expect(&Token::Semicolon)?;
                        Ok(Stmt::Expr(expr))
                    }
                } else {
                    unreachable!()
                }
            }
            _ => {
                let expr = self.parse_expr()?;
                self.expect(&Token::Semicolon)?;
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, GroveError> {
        self.advance(); // consume 'let'
        let name = match self.advance() {
            Token::Ident(s) => s,
            tok => {
                return Err(GroveError::Parse {
                    message: format!("expected identifier, got {}", tok.name()),
                    pos: self.pos,
                })
            }
        };
        self.expect(&Token::Assign)?;
        let expr = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Let(name, expr))
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt, GroveError> {
        self.advance(); // consume 'if'
        let cond = self.parse_expr()?;
        self.expect(&Token::LBrace)?;
        let then_branch = self.parse_block()?;
        self.expect(&Token::RBrace)?;
        let else_branch = if *self.peek() == Token::Else {
            self.advance();
            if *self.peek() == Token::If {
                // else if
                let stmt = self.parse_if_stmt()?;
                vec![stmt]
            } else {
                self.expect(&Token::LBrace)?;
                let stmts = self.parse_block()?;
                self.expect(&Token::RBrace)?;
                stmts
            }
        } else {
            vec![]
        };
        Ok(Stmt::If(cond, then_branch, else_branch))
    }

    fn parse_return(&mut self) -> Result<Stmt, GroveError> {
        self.advance(); // consume 'return'
        let expr = self.parse_expr()?;
        self.expect(&Token::Semicolon)?;
        Ok(Stmt::Return(expr))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, GroveError> {
        let mut stmts = Vec::new();
        while *self.peek() != Token::RBrace && *self.peek() != Token::Eof {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    // Expression parsing with precedence climbing

    fn parse_expr(&mut self) -> Result<Expr, GroveError> {
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, GroveError> {
        let mut left = self.parse_addition()?;
        while let Some(op) = BinOp::from_token(self.peek()) {
            if matches!(op, BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge) {
                self.advance();
                let right = self.parse_addition()?;
                left = Expr::Binary(Box::new(left), op, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, GroveError> {
        let mut left = self.parse_multiplication()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplication()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, GroveError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, GroveError> {
        if *self.peek() == Token::Minus {
            self.advance();
            let expr = self.parse_unary()?;
            Ok(Expr::Unary(UnOp::Neg, Box::new(expr)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, GroveError> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Expr::Literal(n))
            }
            Token::Ident(name) => {
                self.advance();
                if *self.peek() == Token::LParen {
                    self.advance();
                    let mut args = Vec::new();
                    if *self.peek() != Token::RParen {
                        args.push(self.parse_expr()?);
                        while *self.peek() == Token::Comma {
                            self.advance();
                            args.push(self.parse_expr()?);
                        }
                    }
                    self.expect(&Token::RParen)?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Token::If => self.parse_if_expr(),
            tok => Err(GroveError::Parse {
                message: format!("unexpected token in expression: {}", tok.name()),
                pos: self.pos,
            }),
        }
    }

    fn parse_if_expr(&mut self) -> Result<Expr, GroveError> {
        self.advance(); // consume 'if'
        let cond = self.parse_expr()?;
        self.expect(&Token::LBrace)?;
        let then_block = self.parse_block()?;
        self.expect(&Token::RBrace)?;
        let then_expr = if then_block.len() == 1 {
            match &then_block[0] {
                Stmt::Expr(e) => e.clone(),
                Stmt::Return(e) => e.clone(),
                _ => Expr::Literal(0.0),
            }
        } else {
            Expr::Literal(0.0)
        };
        let else_expr = if *self.peek() == Token::Else {
            self.advance();
            if *self.peek() == Token::LBrace {
                self.advance();
                let else_block = self.parse_block()?;
                self.expect(&Token::RBrace)?;
                if else_block.len() == 1 {
                    match &else_block[0] {
                        Stmt::Expr(e) => e.clone(),
                        Stmt::Return(e) => e.clone(),
                        _ => Expr::Literal(0.0),
                    }
                } else {
                    Expr::Literal(0.0)
                }
            } else if *self.peek() == Token::If {
                self.parse_if_expr()?
            } else {
                Expr::Literal(0.0)
            }
        } else {
            Expr::Literal(0.0)
        };
        Ok(Expr::If(Box::new(cond), Box::new(then_expr), Box::new(else_expr)))
    }
}

/// Convenience: lex + parse a string into a Program.
pub fn spring(source: &str) -> Result<Program, GroveError> {
    let tokens = Lexer::new(source).tokenize()?;
    Parser::new(tokens).parse_program()
}
