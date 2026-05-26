//! Compile a Vela AST into `vela-bytecode` IR.
//!
//! The compiler is intentionally small and grows alongside the VM:
//! every construct that compiles here is one the VM can execute. See
//! `docs/ARCHITECTURE.md` for the larger plan.

use vela_bytecode::{Const, ConstIdx, Function, Module, Op, Reg};
use vela_parser::{BinOp, Expr, Lit, Program, Stmt, UnOp, parse_program};

#[derive(Debug, Clone, PartialEq)]
pub struct CompileError {
    pub message: String,
}

impl CompileError {
    fn new(m: impl Into<String>) -> Self {
        Self { message: m.into() }
    }
}

pub fn compile_source(src: &str) -> Result<Module, CompileError> {
    let program =
        parse_program(src).map_err(|e| CompileError::new(format!("parse error: {}", e.message)))?;
    compile_program(&program)
}

pub fn compile_program(program: &Program) -> Result<Module, CompileError> {
    let mut fb = FnBuilder::new("main", 0);
    for (i, stmt) in program.stmts.iter().enumerate() {
        let last = i == program.stmts.len() - 1;
        fb.stmt(stmt, last)?;
    }
    if !matches!(fb.func.code.last(), Some(Op::Return { .. })) {
        let unit = fb.intern(Const::Unit);
        let r = fb.alloc();
        fb.emit(Op::LoadConst { dst: r, k: unit });
        fb.emit(Op::Return { src: r });
    }
    let func = fb.finish();
    Ok(Module {
        functions: vec![func],
        entry: Some(0),
    })
}

struct FnBuilder {
    func: Function,
    locals: Vec<(String, Reg)>,
    next_reg: Reg,
    max_reg: Reg,
}

impl FnBuilder {
    fn new(name: &str, arity: u16) -> Self {
        Self {
            func: Function {
                name: name.into(),
                arity,
                n_regs: 0,
                n_upvals: 0,
                upvals: Vec::new(),
                consts: Vec::new(),
                code: Vec::new(),
                source_path: String::new(),
                source_spans: Vec::new(),
            },
            locals: Vec::new(),
            next_reg: arity,
            max_reg: arity,
        }
    }

    fn finish(mut self) -> Function {
        self.func.n_regs = self.max_reg;
        self.func
    }

    fn intern(&mut self, k: Const) -> ConstIdx {
        if let Some(i) = self.func.consts.iter().position(|c| *c == k) {
            return i as ConstIdx;
        }
        let i = self.func.consts.len() as ConstIdx;
        self.func.consts.push(k);
        i
    }

    fn emit(&mut self, op: Op) {
        self.func.code.push(op);
    }

    fn alloc(&mut self) -> Reg {
        let r = self.next_reg;
        self.next_reg += 1;
        if self.next_reg > self.max_reg {
            self.max_reg = self.next_reg;
        }
        r
    }

    fn resolve_local(&self, name: &str) -> Option<Reg> {
        self.locals
            .iter()
            .rev()
            .find(|(n, _)| n == name)
            .map(|(_, r)| *r)
    }

    fn stmt(&mut self, s: &Stmt, is_last: bool) -> Result<(), CompileError> {
        match s {
            Stmt::Let {
                name,
                params,
                body,
                recursive: _,
                return_ty: _,
            } if params.is_empty() => {
                let r = self.expr(body)?;
                self.locals.push((name.clone(), r));
                Ok(())
            }
            Stmt::Expr(e) => {
                let r = self.expr(e)?;
                if is_last {
                    self.emit(Op::Return { src: r });
                }
                Ok(())
            }
            other => Err(CompileError::new(format!(
                "compiler does not yet handle: {other:?}"
            ))),
        }
    }

    fn expr(&mut self, e: &Expr) -> Result<Reg, CompileError> {
        match e {
            Expr::Lit(l) => self.lit(l),
            Expr::Var(name) => match self.resolve_local(name) {
                Some(r) => Ok(r),
                None => Err(CompileError::new(format!("unbound name: {name}"))),
            },
            Expr::BinOp(op, l, r) => self.binop(*op, l, r),
            Expr::UnaryOp(op, inner) => self.unop(*op, inner),
            Expr::If(c, t, e) => self.if_expr(c, t, e),
            other => Err(CompileError::new(format!(
                "compiler does not yet handle expr: {other:?}"
            ))),
        }
    }

    fn lit(&mut self, l: &Lit) -> Result<Reg, CompileError> {
        let k = match l {
            Lit::Int(n) => Const::Int(*n),
            Lit::UInt(n) => Const::UInt(*n),
            Lit::BigInt(s) => Const::BigInt(s.clone()),
            Lit::Float(f) => Const::Float(*f),
            Lit::Decimal(s) => Const::Decimal(s.clone()),
            Lit::Str(s) => Const::Str(s.clone()),
            Lit::Bool(b) => Const::Bool(*b),
            Lit::Unit => Const::Unit,
        };
        let kidx = self.intern(k);
        let dst = self.alloc();
        self.emit(Op::LoadConst { dst, k: kidx });
        Ok(dst)
    }

    fn binop(&mut self, op: BinOp, l: &Expr, r: &Expr) -> Result<Reg, CompileError> {
        let a = self.expr(l)?;
        let b = self.expr(r)?;
        let dst = self.alloc();
        let op = match op {
            BinOp::Add => Op::Add { dst, a, b },
            BinOp::Sub => Op::Sub { dst, a, b },
            BinOp::Mul => Op::Mul { dst, a, b },
            BinOp::Div => Op::Div { dst, a, b },
            BinOp::Mod => Op::Mod { dst, a, b },
            BinOp::Pow => Op::Pow { dst, a, b },
            BinOp::Concat => Op::Concat { dst, a, b },
            BinOp::Eq => Op::Eq { dst, a, b },
            BinOp::NotEq => Op::Ne { dst, a, b },
            BinOp::Lt => Op::Lt { dst, a, b },
            BinOp::Le => Op::Le { dst, a, b },
            BinOp::Gt => Op::Gt { dst, a, b },
            BinOp::Ge => Op::Ge { dst, a, b },
            other => {
                return Err(CompileError::new(format!(
                    "compiler does not yet handle binop: {other:?}"
                )));
            }
        };
        self.emit(op);
        Ok(dst)
    }

    fn unop(&mut self, op: UnOp, inner: &Expr) -> Result<Reg, CompileError> {
        let a = self.expr(inner)?;
        let dst = self.alloc();
        let op = match op {
            UnOp::Neg => Op::Neg { dst, a },
            UnOp::Not => Op::Not { dst, a },
        };
        self.emit(op);
        Ok(dst)
    }

    fn if_expr(&mut self, c: &Expr, t: &Expr, e: &Expr) -> Result<Reg, CompileError> {
        let cond = self.expr(c)?;
        let jf_idx = self.func.code.len();
        self.emit(Op::JumpIfFalse { cond, offset: 0 });
        let dst = self.alloc();
        let t_val = self.expr(t)?;
        self.emit(Op::Move { dst, src: t_val });
        let jmp_idx = self.func.code.len();
        self.emit(Op::Jump { offset: 0 });
        let else_pc = self.func.code.len() as i32;
        let e_val = self.expr(e)?;
        self.emit(Op::Move { dst, src: e_val });
        let end_pc = self.func.code.len() as i32;
        if let Op::JumpIfFalse { offset, .. } = &mut self.func.code[jf_idx] {
            *offset = else_pc - jf_idx as i32 - 1;
        }
        if let Op::Jump { offset } = &mut self.func.code[jmp_idx] {
            *offset = end_pc - jmp_idx as i32 - 1;
        }
        Ok(dst)
    }
}
