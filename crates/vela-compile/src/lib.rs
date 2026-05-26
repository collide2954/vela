//! Compile a Vela AST into `vela-bytecode` IR.
//!
//! The compiler grows alongside the VM: every construct that compiles
//! here is one the VM can execute. See `docs/ARCHITECTURE.md`.

use vela_bytecode::{Const, ConstIdx, Function, FunctionId, Module, Op, Reg, UpvalDesc, UpvalIdx};
use vela_parser::{BinOp, Expr, Lit, Param, Pat, Program, Stmt, UnOp, parse_program};

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
    let mut ctx = Ctx::new();
    ctx.push_frame("main", 0);
    for (i, stmt) in program.stmts.iter().enumerate() {
        let last = i == program.stmts.len() - 1;
        ctx.stmt(stmt, last)?;
    }
    ctx.ensure_return();
    let main_id = ctx.finish_frame();
    let mut module = ctx.into_module();
    module.entry = Some(main_id);
    Ok(module)
}

#[derive(Debug, Clone)]
struct Local {
    name: String,
    reg: Reg,
}

#[derive(Debug)]
struct Frame {
    func: Function,
    locals: Vec<Local>,
    next_reg: Reg,
    max_reg: Reg,
}

impl Frame {
    fn new(name: &str, arity: u16) -> Self {
        Frame {
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

    fn alloc(&mut self) -> Reg {
        let r = self.next_reg;
        self.next_reg += 1;
        if self.next_reg > self.max_reg {
            self.max_reg = self.next_reg;
        }
        r
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

    fn pc(&self) -> usize {
        self.func.code.len()
    }
}

#[derive(Debug)]
struct Ctx {
    frames: Vec<Frame>,
    finished: Vec<Function>,
}

#[derive(Debug, Clone, Copy)]
enum Resolved {
    Local(Reg),
    Upval(UpvalIdx),
}

impl Ctx {
    fn new() -> Self {
        Self {
            frames: Vec::new(),
            finished: Vec::new(),
        }
    }

    fn push_frame(&mut self, name: &str, arity: u16) {
        self.frames.push(Frame::new(name, arity));
    }

    fn finish_frame(&mut self) -> FunctionId {
        let mut frame = self.frames.pop().expect("no frame to finish");
        frame.func.n_regs = frame.max_reg;
        frame.func.n_upvals = frame.func.upvals.len() as u16;
        let id = self.finished.len() as FunctionId;
        self.finished.push(frame.func);
        id
    }

    fn into_module(self) -> Module {
        Module {
            functions: self.finished,
            entry: None,
        }
    }

    fn cur(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("no frame")
    }

    fn ensure_return(&mut self) {
        let cur = self.cur();
        if !matches!(cur.func.code.last(), Some(Op::Return { .. })) {
            let unit = cur.intern(Const::Unit);
            let r = cur.alloc();
            cur.emit(Op::LoadConst { dst: r, k: unit });
            cur.emit(Op::Return { src: r });
        }
    }

    fn resolve(&mut self, name: &str) -> Option<Resolved> {
        let top = self.frames.len() - 1;
        if let Some(r) = self.frames[top]
            .locals
            .iter()
            .rev()
            .find(|l| l.name == name)
            .map(|l| l.reg)
        {
            return Some(Resolved::Local(r));
        }
        self.resolve_upval(name, top).map(Resolved::Upval)
    }

    fn resolve_upval(&mut self, name: &str, level: usize) -> Option<UpvalIdx> {
        if level == 0 {
            return None;
        }
        let parent = level - 1;
        if let Some(parent_local) = self.frames[parent]
            .locals
            .iter()
            .rev()
            .find(|l| l.name == name)
            .map(|l| l.reg)
        {
            return Some(self.add_upval(level, name, true, parent_local));
        }
        if let Some(parent_upval) = self.resolve_upval(name, parent) {
            return Some(self.add_upval(level, name, false, parent_upval));
        }
        None
    }

    fn add_upval(&mut self, level: usize, _name: &str, from_local: bool, index: u16) -> UpvalIdx {
        let func = &mut self.frames[level].func;
        if let Some(i) = func
            .upvals
            .iter()
            .position(|u| u.from_parent_local == from_local && u.index == index)
        {
            return i as UpvalIdx;
        }
        let i = func.upvals.len() as UpvalIdx;
        func.upvals.push(UpvalDesc {
            from_parent_local: from_local,
            index,
        });
        i
    }

    fn stmt(&mut self, s: &Stmt, is_last: bool) -> Result<(), CompileError> {
        match s {
            Stmt::Let {
                name,
                params,
                body,
                recursive: _,
                return_ty: _,
            } => {
                let r = if params.is_empty() {
                    self.expr(body)?
                } else {
                    self.compile_lambda_chain(params, body)?
                };
                self.cur().locals.push(Local {
                    name: name.clone(),
                    reg: r,
                });
                Ok(())
            }
            Stmt::Expr(e) => {
                let r = self.expr(e)?;
                if is_last {
                    self.cur().emit(Op::Return { src: r });
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
            Expr::Var(name) => self.var(name),
            Expr::BinOp(op, l, r) => self.binop(*op, l, r),
            Expr::UnaryOp(op, inner) => self.unop(*op, inner),
            Expr::If(c, t, e) => self.if_expr(c, t, e),
            Expr::Lambda(params, body) => self.compile_lambda_chain(params, body),
            Expr::App(f, x) => self.app(f, x),
            Expr::Block { stmts, trailing } => self.block(stmts, trailing.as_deref()),
            Expr::Tuple(elems) => self.make_seq(elems, false),
            Expr::Series(elems) => self.make_seq(elems, true),
            Expr::Record(fields) => self.make_record(fields),
            Expr::Field(target, name) => self.field_access(target, name),
            other => Err(CompileError::new(format!(
                "compiler does not yet handle expr: {other:?}"
            ))),
        }
    }

    fn var(&mut self, name: &str) -> Result<Reg, CompileError> {
        match self.resolve(name) {
            Some(Resolved::Local(r)) => Ok(r),
            Some(Resolved::Upval(idx)) => {
                let dst = self.cur().alloc();
                self.cur().emit(Op::GetUpval { dst, idx });
                Ok(dst)
            }
            None => {
                let kidx = self.cur().intern(Const::GlobalName(name.to_string()));
                let cur = self.cur();
                let dst = cur.alloc();
                cur.emit(Op::GetGlobal { dst, name: kidx });
                Ok(dst)
            }
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
        let cur = self.cur();
        let kidx = cur.intern(k);
        let dst = cur.alloc();
        cur.emit(Op::LoadConst { dst, k: kidx });
        Ok(dst)
    }

    fn binop(&mut self, op: BinOp, l: &Expr, r: &Expr) -> Result<Reg, CompileError> {
        let a = self.expr(l)?;
        let b = self.expr(r)?;
        let cur = self.cur();
        let dst = cur.alloc();
        let emit = match op {
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
        cur.emit(emit);
        Ok(dst)
    }

    fn unop(&mut self, op: UnOp, inner: &Expr) -> Result<Reg, CompileError> {
        let a = self.expr(inner)?;
        let cur = self.cur();
        let dst = cur.alloc();
        let emit = match op {
            UnOp::Neg => Op::Neg { dst, a },
            UnOp::Not => Op::Not { dst, a },
        };
        cur.emit(emit);
        Ok(dst)
    }

    fn if_expr(&mut self, c: &Expr, t: &Expr, e: &Expr) -> Result<Reg, CompileError> {
        let cond = self.expr(c)?;
        let jf_idx = self.cur().pc();
        self.cur().emit(Op::JumpIfFalse { cond, offset: 0 });
        let dst = self.cur().alloc();
        let t_val = self.expr(t)?;
        self.cur().emit(Op::Move { dst, src: t_val });
        let jmp_idx = self.cur().pc();
        self.cur().emit(Op::Jump { offset: 0 });
        let else_pc = self.cur().pc() as i32;
        let e_val = self.expr(e)?;
        self.cur().emit(Op::Move { dst, src: e_val });
        let end_pc = self.cur().pc() as i32;
        let cur = self.cur();
        if let Op::JumpIfFalse { offset, .. } = &mut cur.func.code[jf_idx] {
            *offset = else_pc - jf_idx as i32 - 1;
        }
        if let Op::Jump { offset } = &mut cur.func.code[jmp_idx] {
            *offset = end_pc - jmp_idx as i32 - 1;
        }
        Ok(dst)
    }

    fn block(&mut self, stmts: &[Stmt], trailing: Option<&Expr>) -> Result<Reg, CompileError> {
        for s in stmts {
            self.stmt(s, false)?;
        }
        if let Some(t) = trailing {
            self.expr(t)
        } else {
            let cur = self.cur();
            let k = cur.intern(Const::Unit);
            let dst = cur.alloc();
            cur.emit(Op::LoadConst { dst, k });
            Ok(dst)
        }
    }

    fn compile_lambda_chain(&mut self, params: &[Param], body: &Expr) -> Result<Reg, CompileError> {
        if params.is_empty() {
            return Err(CompileError::new("lambda needs at least one parameter"));
        }
        let fid = self.compile_lambda_inner(params, body, "lambda")?;
        let n_upvals = self.finished[fid as usize].upvals.len() as u16;
        let cur = self.cur();
        let dst = cur.alloc();
        cur.emit(Op::MkClosure {
            dst,
            function: fid,
            n_upvals,
        });
        Ok(dst)
    }

    fn compile_lambda_inner(
        &mut self,
        params: &[Param],
        body: &Expr,
        name: &str,
    ) -> Result<FunctionId, CompileError> {
        let head = &params[0];
        let param_name = match &head.pat {
            Pat::Var(n) => n.clone(),
            other => {
                return Err(CompileError::new(format!(
                    "compiler does not yet handle pattern parameter: {other:?}"
                )));
            }
        };
        self.push_frame(name, 1);
        self.cur().locals.push(Local {
            name: param_name,
            reg: 0,
        });
        if params.len() == 1 {
            let r = self.expr(body)?;
            self.cur().emit(Op::Return { src: r });
        } else {
            let inner_fid = self.compile_lambda_inner(&params[1..], body, "lambda")?;
            let n_upvals = self.finished[inner_fid as usize].upvals.len() as u16;
            let cur = self.cur();
            let dst = cur.alloc();
            cur.emit(Op::MkClosure {
                dst,
                function: inner_fid,
                n_upvals,
            });
            cur.emit(Op::Return { src: dst });
        }
        Ok(self.finish_frame())
    }

    fn make_seq(&mut self, elems: &[Expr], series: bool) -> Result<Reg, CompileError> {
        let n = elems.len() as u16;
        let mut regs = Vec::with_capacity(elems.len());
        for e in elems {
            regs.push(self.expr(e)?);
        }
        let cur = self.cur();
        let base = cur.next_reg;
        for r in &regs {
            let slot = cur.alloc();
            cur.emit(Op::Move { dst: slot, src: *r });
        }
        let dst = cur.alloc();
        cur.emit(if series {
            Op::MkSeries { dst, base, n }
        } else {
            Op::MkTuple { dst, base, n }
        });
        Ok(dst)
    }

    fn make_record(&mut self, fields: &[(String, Expr)]) -> Result<Reg, CompileError> {
        let names: Vec<String> = fields.iter().map(|(n, _)| n.clone()).collect();
        let names_idx = self.cur().intern(Const::FieldNames(names));
        let n = fields.len() as u16;
        let mut value_regs = Vec::with_capacity(fields.len());
        for (_, e) in fields {
            value_regs.push(self.expr(e)?);
        }
        let cur = self.cur();
        let base = cur.next_reg;
        for r in &value_regs {
            let slot = cur.alloc();
            cur.emit(Op::Move { dst: slot, src: *r });
        }
        let dst = cur.alloc();
        cur.emit(Op::MkRecord {
            dst,
            base,
            n,
            names: names_idx,
        });
        Ok(dst)
    }

    fn field_access(&mut self, target: &Expr, name: &str) -> Result<Reg, CompileError> {
        let obj = self.expr(target)?;
        let name_idx = self.cur().intern(Const::FieldName(name.to_string()));
        let cur = self.cur();
        let dst = cur.alloc();
        cur.emit(Op::GetField {
            dst,
            obj,
            name: name_idx,
        });
        Ok(dst)
    }

    fn app(&mut self, f: &Expr, arg: &Expr) -> Result<Reg, CompileError> {
        let callee = self.expr(f)?;
        let arg_v = self.expr(arg)?;
        let cur = self.cur();
        let base = cur.alloc();
        cur.emit(Op::Move {
            dst: base,
            src: arg_v,
        });
        let dst = cur.alloc();
        cur.emit(Op::Call {
            dst,
            callee,
            base,
            nargs: 1,
        });
        Ok(dst)
    }
}
