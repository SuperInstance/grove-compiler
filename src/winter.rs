use crate::ast::{ternary, BinOp, Expr, Program, Stmt, TernaryBytecode, TernaryInstruction, Trit, UnOp};

/// Winter: emit balanced ternary bytecode from an optimized AST.
/// Opcodes encoded as trits.
const OP_LOAD_CONST: Trit = Trit::Zero;
const OP_LOAD_VAR: Trit = Trit::Pos;
const OP_ADD: Trit = Trit::Pos;
const OP_SUB: Trit = Trit::Neg;
const OP_MUL: Trit = Trit::Zero;
const OP_DIV: Trit = Trit::Neg;
const OP_NEG: Trit = Trit::Neg;
const OP_RETURN: Trit = Trit::Pos;
const OP_ASSIGN: Trit = Trit::Zero;
const OP_JUMP_IF_ZERO: Trit = Trit::Neg;

/// Emit ternary bytecode from a program.
pub fn winter(program: &Program) -> TernaryBytecode {
    let mut emitter = Emitter::new();
    for stmt in &program.stmts {
        emitter.emit_stmt(stmt);
    }
    emitter.bytecode
}

struct Emitter {
    bytecode: TernaryBytecode,
}

impl Emitter {
    fn new() -> Self {
        Self {
            bytecode: TernaryBytecode::new(),
        }
    }

    fn add_constant(&mut self, val: f64) -> usize {
        let idx = self.bytecode.constants.len();
        self.bytecode.constants.push(val);
        idx
    }

    fn emit(&mut self, opcode: Trit, operand: Vec<Trit>) {
        self.bytecode.instructions.push(TernaryInstruction { opcode, operand });
    }

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(name, expr) => {
                self.emit_expr(expr);
                // Store variable — operand encodes name length as ternary
                let len_trits = ternary::encode_int(name.len() as i64);
                self.emit(OP_ASSIGN, len_trits);
            }
            Stmt::Assign(name, expr) => {
                self.emit_expr(expr);
                let len_trits = ternary::encode_int(name.len() as i64);
                self.emit(OP_ASSIGN, len_trits);
            }
            Stmt::If(cond, then_b, else_b) => {
                self.emit_expr(cond);
                // Jump instruction: encode else-block size
                let else_size = else_b.len() as i64;
                self.emit(OP_JUMP_IF_ZERO, ternary::encode_int(else_size));
                for s in then_b {
                    self.emit_stmt(s);
                }
                for s in else_b {
                    self.emit_stmt(s);
                }
            }
            Stmt::Return(expr) => {
                self.emit_expr(expr);
                self.emit(OP_RETURN, vec![]);
            }
            Stmt::Expr(expr) => {
                self.emit_expr(expr);
            }
        }
    }

    fn emit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(n) => {
                let idx = self.add_constant(*n);
                let idx_trits = ternary::encode_int(idx as i64);
                self.emit(OP_LOAD_CONST, idx_trits);
            }
            Expr::Var(name) => {
                let hash = simple_hash(name);
                let trits = ternary::encode_int(hash);
                self.emit(OP_LOAD_VAR, trits);
            }
            Expr::Binary(left, op, right) => {
                self.emit_expr(left);
                self.emit_expr(right);
                let opcode = match op {
                    BinOp::Add => OP_ADD,
                    BinOp::Sub => OP_SUB,
                    BinOp::Mul => OP_MUL,
                    BinOp::Div => OP_DIV,
                    _ => OP_ADD, // comparison ops default to add for now
                };
                self.emit(opcode, vec![]);
            }
            Expr::Unary(UnOp::Neg, e) => {
                self.emit_expr(e);
                self.emit(OP_NEG, vec![]);
            }
            Expr::If(cond, then_e, else_e) => {
                self.emit_expr(cond);
                self.emit(OP_JUMP_IF_ZERO, vec![Trit::Pos]);
                self.emit_expr(then_e);
                self.emit_expr(else_e);
            }
            Expr::Call(_name, args) => {
                for arg in args {
                    self.emit_expr(arg);
                }
            }
        }
    }
}

/// Simple deterministic hash for variable names.
fn simple_hash(s: &str) -> i64 {
    let mut h: i64 = 5381;
    for b in s.bytes() {
        h = ((h << 5).wrapping_add(h)) ^ b as i64;
    }
    h.abs() % 10000
}
