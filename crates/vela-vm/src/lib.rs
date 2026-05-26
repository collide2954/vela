//! Register-based bytecode VM for the Vela runtime.
//!
//! The VM is the baseline tier: it handles every construct the
//! compiler emits. Hot functions will later be lifted into native code
//! by `vela-jit`; the JIT path returns to the VM at instruction
//! boundaries on bailout. See `docs/ARCHITECTURE.md`.

use std::rc::Rc;
use vela_bytecode::{Const, FunctionId, Module, Op, Reg};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    UInt(u64),
    BigInt(String),
    Float(f64),
    Decimal(String),
    Str(String),
    Bool(bool),
    Sym(String),
    Unit,
    Closure(Rc<Closure>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Closure {
    pub function: FunctionId,
    pub upvalues: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    fn new(m: impl Into<String>) -> Self {
        Self { message: m.into() }
    }
}

pub fn run(module: &Module) -> Result<Value, RuntimeError> {
    let entry = module
        .entry
        .ok_or_else(|| RuntimeError::new("module has no entry function"))?;
    exec(module, entry, &[], &[])
}

fn exec(
    module: &Module,
    fid: FunctionId,
    args: &[Value],
    upvalues: &[Value],
) -> Result<Value, RuntimeError> {
    let f = module.function(fid);
    if args.len() != f.arity as usize {
        return Err(RuntimeError::new(format!(
            "{}: expected {} args, got {}",
            f.name,
            f.arity,
            args.len()
        )));
    }
    let n_regs = f.n_regs.max(f.arity) as usize;
    let mut regs: Vec<Value> = vec![Value::Unit; n_regs.max(1)];
    for (i, v) in args.iter().enumerate() {
        regs[i] = v.clone();
    }
    let mut pc: usize = 0;
    loop {
        let op = f
            .code
            .get(pc)
            .ok_or_else(|| RuntimeError::new(format!("{}: ran off end without Return", f.name)))?;
        match op {
            Op::LoadConst { dst, k } => {
                regs[*dst as usize] = const_to_value(&f.consts[*k as usize]);
            }
            Op::Move { dst, src } => {
                regs[*dst as usize] = regs[*src as usize].clone();
            }
            Op::Add { dst, a, b } => {
                num_binop(&mut regs, *dst, *a, *b, "+", |x, y| x + y, |x, y| x + y)?;
            }
            Op::Sub { dst, a, b } => {
                num_binop(&mut regs, *dst, *a, *b, "-", |x, y| x - y, |x, y| x - y)?;
            }
            Op::Mul { dst, a, b } => {
                num_binop(&mut regs, *dst, *a, *b, "*", |x, y| x * y, |x, y| x * y)?;
            }
            Op::Div { dst, a, b } => {
                num_binop(&mut regs, *dst, *a, *b, "/", |x, y| x / y, |x, y| x / y)?;
            }
            Op::Mod { dst, a, b } => {
                num_binop(&mut regs, *dst, *a, *b, "%", |x, y| x % y, |x, y| x % y)?;
            }
            Op::Pow { dst, a, b } => match (regs[*a as usize].clone(), regs[*b as usize].clone()) {
                (Value::Int(x), Value::Int(y)) if y >= 0 => {
                    regs[*dst as usize] = Value::Int(x.pow(y as u32));
                }
                (Value::Float(x), Value::Float(y)) => {
                    regs[*dst as usize] = Value::Float(x.powf(y));
                }
                (x, y) => return Err(RuntimeError::new(format!("^: {x:?} ^ {y:?}"))),
            },
            Op::Concat { dst, a, b } => {
                match (regs[*a as usize].clone(), regs[*b as usize].clone()) {
                    (Value::Str(x), Value::Str(y)) => regs[*dst as usize] = Value::Str(x + &y),
                    (x, y) => return Err(RuntimeError::new(format!("++: {x:?} ++ {y:?}"))),
                }
            }
            Op::Eq { dst, a, b } => {
                regs[*dst as usize] = Value::Bool(regs[*a as usize] == regs[*b as usize]);
            }
            Op::Ne { dst, a, b } => {
                regs[*dst as usize] = Value::Bool(regs[*a as usize] != regs[*b as usize]);
            }
            Op::Lt { dst, a, b } => {
                cmp(&mut regs, *dst, *a, *b, |o| o == std::cmp::Ordering::Less)?
            }
            Op::Le { dst, a, b } => cmp(&mut regs, *dst, *a, *b, |o| {
                o != std::cmp::Ordering::Greater
            })?,
            Op::Gt { dst, a, b } => cmp(&mut regs, *dst, *a, *b, |o| {
                o == std::cmp::Ordering::Greater
            })?,
            Op::Ge { dst, a, b } => {
                cmp(&mut regs, *dst, *a, *b, |o| o != std::cmp::Ordering::Less)?
            }
            Op::Neg { dst, a } => match regs[*a as usize].clone() {
                Value::Int(n) => regs[*dst as usize] = Value::Int(-n),
                Value::Float(n) => regs[*dst as usize] = Value::Float(-n),
                other => return Err(RuntimeError::new(format!("neg: {other:?}"))),
            },
            Op::Not { dst, a } => match regs[*a as usize].clone() {
                Value::Bool(b) => regs[*dst as usize] = Value::Bool(!b),
                other => return Err(RuntimeError::new(format!("not: {other:?}"))),
            },
            Op::Jump { offset } => {
                pc = ((pc as i32) + 1 + offset) as usize;
                continue;
            }
            Op::JumpIfFalse { cond, offset } => {
                let go = match &regs[*cond as usize] {
                    Value::Bool(b) => !*b,
                    other => {
                        return Err(RuntimeError::new(format!(
                            "JumpIfFalse condition must be Bool, got {other:?}"
                        )));
                    }
                };
                if go {
                    pc = ((pc as i32) + 1 + offset) as usize;
                    continue;
                }
            }
            Op::Return { src } => return Ok(regs[*src as usize].clone()),
            Op::MkClosure {
                dst,
                function,
                n_upvals: _,
            } => {
                let descs = &module.function(*function).upvals;
                let mut caps = Vec::with_capacity(descs.len());
                for d in descs {
                    if d.from_parent_local {
                        caps.push(regs[d.index as usize].clone());
                    } else {
                        caps.push(upvalues[d.index as usize].clone());
                    }
                }
                regs[*dst as usize] = Value::Closure(Rc::new(Closure {
                    function: *function,
                    upvalues: caps,
                }));
            }
            Op::GetUpval { dst, idx } => {
                regs[*dst as usize] = upvalues[*idx as usize].clone();
            }
            Op::Call {
                dst,
                callee,
                base,
                nargs,
            } => {
                let f_v = regs[*callee as usize].clone();
                let mut a = Vec::with_capacity(*nargs as usize);
                for i in 0..*nargs {
                    a.push(regs[(*base + i) as usize].clone());
                }
                let result = call_value(module, &f_v, &a)?;
                regs[*dst as usize] = result;
            }
            other => {
                return Err(RuntimeError::new(format!(
                    "VM does not yet handle: {other:?}"
                )));
            }
        }
        pc += 1;
    }
}

fn call_value(module: &Module, f: &Value, args: &[Value]) -> Result<Value, RuntimeError> {
    match f {
        Value::Closure(c) => exec(module, c.function, args, &c.upvalues),
        other => Err(RuntimeError::new(format!("not callable: {other:?}"))),
    }
}

fn const_to_value(k: &Const) -> Value {
    match k {
        Const::Int(n) => Value::Int(*n),
        Const::UInt(n) => Value::UInt(*n),
        Const::BigInt(s) => Value::BigInt(s.clone()),
        Const::Float(f) => Value::Float(*f),
        Const::Decimal(s) => Value::Decimal(s.clone()),
        Const::Str(s) => Value::Str(s.clone()),
        Const::Bool(b) => Value::Bool(*b),
        Const::Sym(s) => Value::Sym(s.clone()),
        Const::Unit => Value::Unit,
        Const::FieldName(_) | Const::CtorName(_) | Const::GlobalName(_) => Value::Unit,
    }
}

fn num_binop(
    regs: &mut [Value],
    dst: Reg,
    a: Reg,
    b: Reg,
    sym: &str,
    iop: fn(i64, i64) -> i64,
    fop: fn(f64, f64) -> f64,
) -> Result<(), RuntimeError> {
    let r = match (regs[a as usize].clone(), regs[b as usize].clone()) {
        (Value::Int(x), Value::Int(y)) => Value::Int(iop(x, y)),
        (Value::Float(x), Value::Float(y)) => Value::Float(fop(x, y)),
        (x, y) => {
            return Err(RuntimeError::new(format!("{sym}: {x:?} {sym} {y:?}")));
        }
    };
    regs[dst as usize] = r;
    Ok(())
}

fn cmp(
    regs: &mut [Value],
    dst: Reg,
    a: Reg,
    b: Reg,
    pred: fn(std::cmp::Ordering) -> bool,
) -> Result<(), RuntimeError> {
    let ord = match (&regs[a as usize], &regs[b as usize]) {
        (Value::Int(x), Value::Int(y)) => x.cmp(y),
        (Value::Float(x), Value::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
        (Value::Str(x), Value::Str(y)) => x.cmp(y),
        (Value::Bool(x), Value::Bool(y)) => x.cmp(y),
        (x, y) => {
            return Err(RuntimeError::new(format!("cmp: {x:?} <> {y:?}")));
        }
    };
    regs[dst as usize] = Value::Bool(pred(ord));
    Ok(())
}
