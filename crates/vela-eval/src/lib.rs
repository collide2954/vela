//! Tree-walking evaluator for the Vela language.

use std::collections::HashMap;
use std::rc::Rc;
use vela_parser::{BinOp, Expr, Lit, Param, Stmt, UnOp, parse_program};

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Sym(String),
    Unit,
    Tuple(Vec<Value>),
    Series(Vec<Value>),
    Record(Vec<(String, Value)>),
    Cons(String, Vec<Value>),
    Closure { params: Vec<String>, body: Expr, env: Env },
    Builtin(BuiltinFn),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Sym(a), Value::Sym(b)) => a == b,
            (Value::Unit, Value::Unit) => true,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Series(a), Value::Series(b)) => a == b,
            (Value::Record(a), Value::Record(b)) => a == b,
            (Value::Cons(an, aa), Value::Cons(bn, ba)) => an == bn && aa == ba,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct BuiltinFn(pub Rc<dyn Fn(Value) -> Result<Value, RuntimeError>>);

impl std::fmt::Debug for BuiltinFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<builtin>")
    }
}

#[derive(Debug, Clone, Default)]
pub struct Env {
    bindings: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        Self::default()
    }

    fn extend(&self, name: String, value: Value) -> Env {
        let mut bindings = self.bindings.clone();
        bindings.insert(name, value);
        Env { bindings }
    }

    fn lookup(&self, name: &str) -> Option<&Value> {
        self.bindings.get(name)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into() }
    }
}

pub fn run(src: &str) -> Result<Value, RuntimeError> {
    let program = parse_program(src)
        .map_err(|e| RuntimeError::new(format!("parse error: {}", e.message)))?;
    let mut env = prelude();
    let mut last = Value::Unit;
    for stmt in &program.stmts {
        last = eval_stmt(stmt, &mut env)?;
    }
    Ok(last)
}

fn prelude() -> Env {
    let mut env = Env::new();
    env = env.extend(
        "println".into(),
        Value::Builtin(BuiltinFn(Rc::new(|v| {
            println!("{}", show(&v));
            Ok(Value::Unit)
        }))),
    );
    env = env.extend(
        "print".into(),
        Value::Builtin(BuiltinFn(Rc::new(|v| {
            print!("{}", show(&v));
            Ok(Value::Unit)
        }))),
    );
    env
}

pub fn show(v: &Value) -> String {
    match v {
        Value::Int(n) => n.to_string(),
        Value::Float(f) => {
            if f.fract() == 0.0 && f.is_finite() {
                format!("{f:.1}")
            } else {
                f.to_string()
            }
        }
        Value::Str(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Sym(s) => format!(":{s}"),
        Value::Unit => "()".into(),
        Value::Tuple(vs) => {
            let parts: Vec<String> = vs.iter().map(show).collect();
            format!("({})", parts.join(", "))
        }
        Value::Series(vs) => {
            let parts: Vec<String> = vs.iter().map(show).collect();
            format!("[{}]", parts.join(", "))
        }
        Value::Record(fs) => {
            let parts: Vec<String> =
                fs.iter().map(|(n, v)| format!("{n} = {}", show(v))).collect();
            format!("{{ {} }}", parts.join(", "))
        }
        Value::Cons(name, args) => {
            if args.is_empty() {
                name.clone()
            } else {
                let parts: Vec<String> = args.iter().map(show).collect();
                format!("{name} {}", parts.join(" "))
            }
        }
        Value::Closure { .. } | Value::Builtin(_) => "<fn>".into(),
    }
}

fn eval_stmt(stmt: &Stmt, env: &mut Env) -> Result<Value, RuntimeError> {
    match stmt {
        Stmt::Let { name, params, body, .. } => {
            let value = if params.is_empty() {
                eval(body, env)?
            } else {
                make_closure(params, body, env)
            };
            *env = env.extend(name.clone(), value);
            Ok(Value::Unit)
        }
        Stmt::Var { name, body, .. } => {
            let value = eval(body, env)?;
            *env = env.extend(name.clone(), value);
            Ok(Value::Unit)
        }
        Stmt::Expr(e) => eval(e, env),
        other => Err(RuntimeError::new(format!("cannot yet evaluate {other:?}"))),
    }
}

fn make_closure(params: &[Param], body: &Expr, env: &Env) -> Value {
    Value::Closure {
        params: params.iter().map(|p| p.name.clone()).collect(),
        body: body.clone(),
        env: env.clone(),
    }
}

fn eval(expr: &Expr, env: &Env) -> Result<Value, RuntimeError> {
    match expr {
        Expr::Lit(Lit::Int(n)) => Ok(Value::Int(*n)),
        Expr::Lit(Lit::Float(f)) => Ok(Value::Float(*f)),
        Expr::Lit(Lit::Str(s)) => Ok(Value::Str(s.clone())),
        Expr::Lit(Lit::Bool(b)) => Ok(Value::Bool(*b)),
        Expr::Lit(Lit::Unit) => Ok(Value::Unit),
        Expr::Sym(s) => Ok(Value::Sym(s.clone())),
        Expr::Var(name) => env
            .lookup(name)
            .cloned()
            .ok_or_else(|| RuntimeError::new(format!("unbound: {name}"))),
        Expr::Lambda(params, body) => Ok(Value::Closure {
            params: params.clone(),
            body: (**body).clone(),
            env: env.clone(),
        }),
        Expr::App(f, x) => {
            let fv = eval(f, env)?;
            let xv = eval(x, env)?;
            apply(fv, xv)
        }
        Expr::If(c, t, e) => {
            let cv = eval(c, env)?;
            match cv {
                Value::Bool(true) => eval(t, env),
                Value::Bool(false) => eval(e, env),
                other => Err(RuntimeError::new(format!(
                    "if condition must be Bool, got {}",
                    show(&other)
                ))),
            }
        }
        Expr::BinOp(op, l, r) => eval_binop(*op, l, r, env),
        Expr::UnaryOp(op, inner) => eval_unop(*op, inner, env),
        Expr::Tuple(elems) => {
            let mut vs = Vec::with_capacity(elems.len());
            for e in elems {
                vs.push(eval(e, env)?);
            }
            Ok(Value::Tuple(vs))
        }
        Expr::Series(elems) => {
            let mut vs = Vec::with_capacity(elems.len());
            for e in elems {
                vs.push(eval(e, env)?);
            }
            Ok(Value::Series(vs))
        }
        Expr::Record(fields) => {
            let mut vs = Vec::with_capacity(fields.len());
            for (n, e) in fields {
                vs.push((n.clone(), eval(e, env)?));
            }
            Ok(Value::Record(vs))
        }
        Expr::Field(target, name) => {
            let v = eval(target, env)?;
            match v {
                Value::Record(fs) => fs
                    .into_iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, v)| v)
                    .ok_or_else(|| RuntimeError::new(format!("no field {name}"))),
                other => Err(RuntimeError::new(format!(
                    "field access requires a record, got {}",
                    show(&other)
                ))),
            }
        }
        Expr::Block { stmts, trailing } => {
            let mut block_env = env.clone();
            for s in stmts {
                eval_stmt(s, &mut block_env)?;
            }
            match trailing {
                Some(e) => eval(e, &block_env),
                None => Ok(Value::Unit),
            }
        }
        other => Err(RuntimeError::new(format!("cannot yet evaluate {other:?}"))),
    }
}

fn apply(f: Value, arg: Value) -> Result<Value, RuntimeError> {
    match f {
        Value::Closure { params, body, env } => {
            if params.is_empty() {
                return Err(RuntimeError::new("calling a zero-parameter closure"));
            }
            let inner_env = env.extend(params[0].clone(), arg);
            if params.len() == 1 {
                eval(&body, &inner_env)
            } else {
                Ok(Value::Closure {
                    params: params[1..].to_vec(),
                    body,
                    env: inner_env,
                })
            }
        }
        Value::Builtin(BuiltinFn(f)) => f(arg),
        other => Err(RuntimeError::new(format!(
            "calling a non-function: {}",
            show(&other)
        ))),
    }
}

fn eval_binop(op: BinOp, l: &Expr, r: &Expr, env: &Env) -> Result<Value, RuntimeError> {
    if matches!(op, BinOp::Pipe) {
        let lv = eval(l, env)?;
        let rv = eval(r, env)?;
        return apply(rv, lv);
    }
    let lv = eval(l, env)?;
    let rv = eval(r, env)?;
    match (op, &lv, &rv) {
        (BinOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
        (BinOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
        (BinOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
        (BinOp::Div, Value::Int(a), Value::Int(b)) if *b != 0 => Ok(Value::Int(a / b)),
        (BinOp::Mod, Value::Int(a), Value::Int(b)) if *b != 0 => Ok(Value::Int(a % b)),
        (BinOp::Pow, Value::Int(a), Value::Int(b)) if *b >= 0 => {
            Ok(Value::Int(a.pow(*b as u32)))
        }
        (BinOp::Add, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
        (BinOp::Sub, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
        (BinOp::Mul, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
        (BinOp::Div, Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
        (BinOp::Eq, a, b) => Ok(Value::Bool(a == b)),
        (BinOp::NotEq, a, b) => Ok(Value::Bool(a != b)),
        (BinOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
        (BinOp::Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
        (BinOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
        (BinOp::Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
        (BinOp::Lt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
        (BinOp::Le, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
        (BinOp::Gt, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
        (BinOp::Ge, Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
        (BinOp::And, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
        (BinOp::Or, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
        (BinOp::Concat, Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
        (BinOp::Concat, Value::Series(a), Value::Series(b)) => {
            let mut out = a.clone();
            out.extend(b.iter().cloned());
            Ok(Value::Series(out))
        }
        _ => Err(RuntimeError::new(format!(
            "operator {op:?} not supported on {} and {}",
            show(&lv),
            show(&rv)
        ))),
    }
}

fn eval_unop(op: UnOp, inner: &Expr, env: &Env) -> Result<Value, RuntimeError> {
    let v = eval(inner, env)?;
    match (op, v) {
        (UnOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
        (UnOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
        (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
        (_, other) => Err(RuntimeError::new(format!(
            "unary {op:?} not supported on {}",
            show(&other)
        ))),
    }
}
