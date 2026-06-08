#[cfg(test)]
mod tests {
    use grove_compiler::*;

    // ── Lexing tests ──────────────────────────────────────────────────────

    #[test]
    fn lex_number() {
        let tokens = spring::Lexer::new("42").tokenize().unwrap();
        assert_eq!(tokens[0], Token::Number(42.0));
    }

    #[test]
    fn lex_float() {
        let tokens = spring::Lexer::new("3.14").tokenize().unwrap();
        assert_eq!(tokens[0], Token::Number(3.14));
    }

    #[test]
    fn lex_ident() {
        let tokens = spring::Lexer::new("foo").tokenize().unwrap();
        assert_eq!(tokens[0], Token::Ident("foo".into()));
    }

    #[test]
    fn lex_keywords() {
        let tokens = spring::Lexer::new("let if else fn return").tokenize().unwrap();
        assert_eq!(tokens[0], Token::Let);
        assert_eq!(tokens[1], Token::If);
        assert_eq!(tokens[2], Token::Else);
        assert_eq!(tokens[3], Token::Fn);
        assert_eq!(tokens[4], Token::Return);
    }

    #[test]
    fn lex_operators() {
        let tokens = spring::Lexer::new("+ - * / == != < > <= >=").tokenize().unwrap();
        assert_eq!(tokens[0], Token::Plus);
        assert_eq!(tokens[1], Token::Minus);
        assert_eq!(tokens[2], Token::Star);
        assert_eq!(tokens[3], Token::Slash);
        assert_eq!(tokens[4], Token::Eq);
        assert_eq!(tokens[5], Token::Neq);
        assert_eq!(tokens[6], Token::Lt);
        assert_eq!(tokens[7], Token::Gt);
        assert_eq!(tokens[8], Token::Le);
        assert_eq!(tokens[9], Token::Ge);
    }

    #[test]
    fn lex_punctuation() {
        let tokens = spring::Lexer::new("( ) { } ; , =").tokenize().unwrap();
        assert_eq!(tokens[0], Token::LParen);
        assert_eq!(tokens[1], Token::RParen);
        assert_eq!(tokens[2], Token::LBrace);
        assert_eq!(tokens[3], Token::RBrace);
        assert_eq!(tokens[4], Token::Semicolon);
        assert_eq!(tokens[5], Token::Comma);
        assert_eq!(tokens[6], Token::Assign);
    }

    #[test]
    fn lex_eof() {
        let tokens = spring::Lexer::new("").tokenize().unwrap();
        assert_eq!(tokens[0], Token::Eof);
    }

    #[test]
    fn lex_unexpected_char() {
        let result = spring::Lexer::new("@").tokenize();
        assert!(result.is_err());
    }

    #[test]
    fn lex_bang_without_eq() {
        let result = spring::Lexer::new("!").tokenize();
        assert!(result.is_err());
    }

    // ── Parsing tests ─────────────────────────────────────────────────────

    #[test]
    fn parse_let_statement() {
        let prog = spring("let x = 5;").unwrap();
        assert_eq!(prog.stmts.len(), 1);
        match &prog.stmts[0] {
            Stmt::Let(name, _) => assert_eq!(name, "x"),
            _ => panic!("expected Let statement"),
        }
    }

    #[test]
    fn parse_binary_expr() {
        let prog = spring("let x = 2 + 3;").unwrap();
        match &prog.stmts[0] {
            Stmt::Let(_, Expr::Binary(_, BinOp::Add, _)) => {}
            _ => panic!("expected binary add"),
        }
    }

    #[test]
    fn parse_precedence() {
        // 2 + 3 * 4 should parse as 2 + (3 * 4)
        let prog = spring("let x = 2 + 3 * 4;").unwrap();
        match &prog.stmts[0] {
            Stmt::Let(_, Expr::Binary(_, BinOp::Add, right)) => {
                match right.as_ref() {
                    Expr::Binary(_, BinOp::Mul, _) => {}
                    _ => panic!("expected mul on right side"),
                }
            }
            _ => panic!("expected binary add"),
        }
    }

    #[test]
    fn parse_unary() {
        let prog = spring("let x = -5;").unwrap();
        match &prog.stmts[0] {
            Stmt::Let(_, Expr::Unary(ast::UnOp::Neg, _)) => {}
            _ => panic!("expected unary neg"),
        }
    }

    #[test]
    fn parse_if_statement() {
        let prog = spring("if x { let y = 1; } else { let z = 2; }").unwrap();
        match &prog.stmts[0] {
            Stmt::If(_, then_b, else_b) => {
                assert_eq!(then_b.len(), 1);
                assert_eq!(else_b.len(), 1);
            }
            _ => panic!("expected If statement"),
        }
    }

    #[test]
    fn parse_function_call() {
        let prog = spring("let x = foo(1, 2);").unwrap();
        match &prog.stmts[0] {
            Stmt::Let(_, Expr::Call(name, args)) => {
                assert_eq!(name, "foo");
                assert_eq!(args.len(), 2);
            }
            _ => panic!("expected Call expression"),
        }
    }

    #[test]
    fn parse_assignment() {
        let prog = spring("x = 10;").unwrap();
        match &prog.stmts[0] {
            Stmt::Assign(name, _) => assert_eq!(name, "x"),
            _ => panic!("expected Assign"),
        }
    }

    #[test]
    fn parse_return() {
        let prog = spring("return 42;").unwrap();
        match &prog.stmts[0] {
            Stmt::Return(_) => {}
            _ => panic!("expected Return"),
        }
    }

    #[test]
    fn parse_parens() {
        let prog = spring("let x = (2 + 3) * 4;").unwrap();
        match &prog.stmts[0] {
            Stmt::Let(_, Expr::Binary(left, BinOp::Mul, _)) => {
                match left.as_ref() {
                    Expr::Binary(_, BinOp::Add, _) => {}
                    _ => panic!("expected add inside parens"),
                }
            }
            _ => panic!("expected binary mul"),
        }
    }

    // ── Type checking (Summer) tests ──────────────────────────────────────

    #[test]
    fn summer_valid_program() {
        let prog = spring("let x = 5; let y = x + 1;").unwrap();
        assert!(summer(&prog).is_ok());
    }

    #[test]
    fn summer_undefined_variable() {
        let prog = spring("let x = y + 1;").unwrap();
        assert!(summer(&prog).is_err());
    }

    #[test]
    fn summer_assign_undefined() {
        let prog = spring("x = 5;").unwrap();
        assert!(summer(&prog).is_err());
    }

    #[test]
    fn summer_scope_in_if() {
        let prog = spring("let x = 1; if x { let y = 2; } else { let z = 3; }").unwrap();
        assert!(summer(&prog).is_ok());
    }

    #[test]
    fn summer_nested_let() {
        let prog = spring("let a = 1; let b = a + 2; let c = a + b;").unwrap();
        assert!(summer(&prog).is_ok());
    }

    // ── Const eval tests ──────────────────────────────────────────────────

    #[test]
    fn const_eval_literal() {
        assert_eq!(summer::const_eval(&Expr::Literal(42.0)), Some(42.0));
    }

    #[test]
    fn const_eval_binary() {
        use ast::BinOp;
        let expr = Expr::Binary(Box::new(Expr::Literal(2.0)), BinOp::Add, Box::new(Expr::Literal(3.0)));
        assert_eq!(summer::const_eval(&expr), Some(5.0));
    }

    // ── Optimization (Autumn) tests ───────────────────────────────────────

    #[test]
    fn autumn_constant_fold() {
        let mut prog = spring("let x = 2 + 3;").unwrap();
        autumn(&mut prog);
        match &prog.stmts[0] {
            Stmt::Let(_, Expr::Literal(5.0)) => {}
            Stmt::Let(_, e) => panic!("expected Literal(5.0), got {e:?}"),
            _ => panic!("expected Let"),
        }
    }

    #[test]
    fn autumn_identity_add_zero() {
        let mut prog = spring("let x = y + 0;").unwrap();
        autumn(&mut prog);
        match &prog.stmts[0] {
            Stmt::Let(_, Expr::Var(name)) => assert_eq!(name, "y"),
            Stmt::Let(_, e) => panic!("expected Var, got {e:?}"),
            _ => panic!("expected Let"),
        }
    }

    #[test]
    fn autumn_identity_mul_one() {
        let mut prog = spring("let x = y * 1;").unwrap();
        autumn(&mut prog);
        match &prog.stmts[0] {
            Stmt::Let(_, Expr::Var(name)) => assert_eq!(name, "y"),
            _ => panic!("expected Var after mul-by-1 elimination"),
        }
    }

    #[test]
    fn autumn_strength_reduce_mul2() {
        let mut prog = spring("let x = y * 2;").unwrap();
        autumn(&mut prog);
        match &prog.stmts[0] {
            Stmt::Let(_, Expr::Binary(_, BinOp::Add, _)) => {}
            Stmt::Let(_, e) => panic!("expected add after strength reduction, got {e:?}"),
            _ => panic!("expected Let"),
        }
    }

    #[test]
    fn autumn_negate_literal() {
        let mut prog = spring("let x = -5;").unwrap();
        autumn(&mut prog);
        match &prog.stmts[0] {
            Stmt::Let(_, Expr::Literal(n)) => assert_eq!(*n, -5.0),
            _ => panic!("expected folded neg literal"),
        }
    }

    #[test]
    fn autumn_div_by_one() {
        let mut prog = spring("let x = y / 1;").unwrap();
        autumn(&mut prog);
        match &prog.stmts[0] {
            Stmt::Let(_, Expr::Var(name)) => assert_eq!(name, "y"),
            _ => panic!("expected Var after div-by-1"),
        }
    }

    #[test]
    fn autumn_sub_zero() {
        let mut prog = spring("let x = y - 0;").unwrap();
        autumn(&mut prog);
        match &prog.stmts[0] {
            Stmt::Let(_, Expr::Var(name)) => assert_eq!(name, "y"),
            _ => panic!("expected Var after sub-zero"),
        }
    }

    // ── Ternary encoding tests ────────────────────────────────────────────

    #[test]
    fn ternary_encode_zero() {
        let trits = ast::ternary::encode_int(0);
        assert_eq!(trits, vec![Trit::Zero]);
    }

    #[test]
    fn ternary_encode_one() {
        let trits = ast::ternary::encode_int(1);
        assert_eq!(trits, vec![Trit::Pos]);
    }

    #[test]
    fn ternary_encode_neg_one() {
        let trits = ast::ternary::encode_int(-1);
        assert_eq!(trits, vec![Trit::Neg]);
    }

    #[test]
    fn ternary_roundtrip() {
        for n in [-100i64, -5, -1, 0, 1, 2, 5, 42, 100, 999] {
            let trits = ast::ternary::encode_int(n);
            let decoded = ast::ternary::decode_int(&trits);
            assert_eq!(decoded, n, "roundtrip failed for {n}");
        }
    }

    #[test]
    fn trit_value() {
        assert_eq!(Trit::Neg.value(), -1);
        assert_eq!(Trit::Zero.value(), 0);
        assert_eq!(Trit::Pos.value(), 1);
    }

    // ── Winter emission tests ─────────────────────────────────────────────

    #[test]
    fn winter_emit_literal() {
        let prog = spring("let x = 42;").unwrap();
        let bc = winter(&prog);
        assert!(!bc.instructions.is_empty());
        assert!(bc.constants.contains(&42.0));
    }

    #[test]
    fn winter_emit_binary() {
        let prog = spring("let x = 2 + 3;").unwrap();
        let bc = winter(&prog);
        // Should have load_const, load_const, add, assign
        assert!(bc.instructions.len() >= 3);
    }

    #[test]
    fn winter_emit_constants() {
        let prog = spring("let a = 1; let b = 2; let c = 3;").unwrap();
        let bc = winter(&prog);
        assert!(bc.constants.contains(&1.0));
        assert!(bc.constants.contains(&2.0));
        assert!(bc.constants.contains(&3.0));
    }

    #[test]
    fn winter_emit_unary() {
        let prog = spring("let x = -5;").unwrap();
        let bc = winter(&prog);
        assert!(bc.instructions.len() >= 2);
    }

    // ── Full pipeline tests ───────────────────────────────────────────────

    #[test]
    fn full_pipeline_simple() {
        let source = "let x = 10; let y = x + 5;";
        let prog = spring(source).unwrap();
        assert!(summer(&prog).is_ok());
        let mut opt_prog = prog.clone();
        autumn(&mut opt_prog);
        let _bc = winter(&opt_prog);
    }

    #[test]
    fn full_pipeline_optimized() {
        let source = "let x = 2 + 3 * 4;";
        let mut prog = spring(source).unwrap();
        summer(&prog).unwrap();
        autumn(&mut prog);
        // After optimization, the expression should be folded
        let bc = winter(&prog);
        assert!(bc.constants.contains(&14.0));
    }
}
