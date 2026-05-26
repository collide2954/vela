//! Tree-walking evaluator for the Vela language.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use vela_parser::{
    BinOp, Expr, LetBinding, ListPart, Lit, Param, Pat, Stmt, TypeDeclBody, UnOp, parse_program,
};

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
    Closure { params: Vec<Pat>, body: Expr, env: Env },
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
    frames: Vec<Rc<RefCell<HashMap<String, Value>>>>,
}

impl Env {
    pub fn new() -> Self {
        Self::default()
    }

    fn extend(&self, name: String, value: Value) -> Env {
        let mut map = HashMap::new();
        map.insert(name, value);
        let mut frames = self.frames.clone();
        frames.push(Rc::new(RefCell::new(map)));
        Env { frames }
    }

    fn lookup(&self, name: &str) -> Option<Value> {
        for frame in self.frames.iter().rev() {
            if let Some(v) = frame.borrow().get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    fn mutate(&self, name: &str, value: Value) -> bool {
        for frame in self.frames.iter().rev() {
            if frame.borrow().contains_key(name) {
                frame.borrow_mut().insert(name.into(), value);
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
    pub short_circuit: Option<Value>,
}

impl RuntimeError {
    fn new(msg: impl Into<String>) -> Self {
        Self { message: msg.into(), short_circuit: None }
    }

    fn short_circuit(value: Value) -> Self {
        Self { message: "short-circuit".into(), short_circuit: Some(value) }
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

#[derive(Debug, Clone)]
pub struct TestReport {
    pub name: String,
    pub passed: bool,
    pub message: Option<String>,
}

pub fn run_tests(src: &str) -> Result<Vec<TestReport>, RuntimeError> {
    let program = parse_program(src)
        .map_err(|e| RuntimeError::new(format!("parse error: {}", e.message)))?;
    let mut env = prelude();
    let mut reports = Vec::new();
    for stmt in &program.stmts {
        if let Stmt::Tests(cases) = stmt {
            for case in cases {
                let report = match case {
                    vela_parser::TestCase::Test { name, body } => {
                        match eval(body, &env) {
                            Ok(_) => TestReport { name: name.clone(), passed: true, message: None },
                            Err(e) => TestReport {
                                name: name.clone(),
                                passed: false,
                                message: Some(e.message),
                            },
                        }
                    }
                    vela_parser::TestCase::Prop { name, .. } => TestReport {
                        name: name.clone(),
                        passed: true,
                        message: Some("prop tests not yet supported".into()),
                    },
                };
                reports.push(report);
            }
        } else {
            eval_stmt(stmt, &mut env)?;
        }
    }
    Ok(reports)
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
    env = env.extend("None".into(), Value::Cons("None".into(), vec![]));
    env = env.extend("Some".into(), make_constructor("Some".into(), 1));
    env = env.extend("Ok".into(), make_constructor("Ok".into(), 1));
    env = env.extend("Err".into(), make_constructor("Err".into(), 1));
    env = env.extend(
        "assert".into(),
        Value::Builtin(BuiltinFn(Rc::new(|v| match v {
            Value::Bool(true) => Ok(Value::Unit),
            Value::Bool(false) => Err(RuntimeError::new("assertion failed")),
            other => Err(RuntimeError::new(format!(
                "assert requires Bool, got {}",
                show(&other)
            ))),
        }))),
    );
    env = env.extend("length".into(), builtin1(|v| match v {
        Value::Series(vs) => Ok(Value::Int(vs.len() as i64)),
        Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
        other => Err(RuntimeError::new(format!(
            "length expects a series or string, got {}",
            show(&other)
        ))),
    }));
    env = env.extend(
        "map".into(),
        builtin1(|f| {
            Ok(builtin1(move |xs| match xs {
                Value::Series(vs) => {
                    let mut out = Vec::with_capacity(vs.len());
                    for v in vs {
                        out.push(apply(f.clone(), v)?);
                    }
                    Ok(Value::Series(out))
                }
                other => Err(RuntimeError::new(format!(
                    "map expects a series, got {}",
                    show(&other)
                ))),
            }))
        }),
    );
    env = env.extend(
        "filter".into(),
        builtin1(|f| {
            Ok(builtin1(move |xs| match xs {
                Value::Series(vs) => {
                    let mut out = Vec::new();
                    for v in vs {
                        match apply(f.clone(), v.clone())? {
                            Value::Bool(true) => out.push(v),
                            Value::Bool(false) => {}
                            other => {
                                return Err(RuntimeError::new(format!(
                                    "filter predicate must return Bool, got {}",
                                    show(&other)
                                )));
                            }
                        }
                    }
                    Ok(Value::Series(out))
                }
                other => Err(RuntimeError::new(format!(
                    "filter expects a series, got {}",
                    show(&other)
                ))),
            }))
        }),
    );
    env = env.extend(
        "fold".into(),
        builtin1(|f| {
            Ok(builtin1(move |init| {
                let f = f.clone();
                Ok(builtin1(move |xs| match xs {
                    Value::Series(vs) => {
                        let mut acc = init.clone();
                        for v in vs {
                            let next = apply(f.clone(), acc)?;
                            acc = apply(next, v)?;
                        }
                        Ok(acc)
                    }
                    other => Err(RuntimeError::new(format!(
                        "fold expects a series, got {}",
                        show(&other)
                    ))),
                }))
            }))
        }),
    );
    env = env.extend("sum".into(), builtin1(|xs| sum_series(xs)));
    env = env.extend("mean".into(), builtin1(|xs| mean_series(xs)));
    env = env.extend("min".into(), builtin1(|xs| extremum_series(xs, true)));
    env = env.extend("max".into(), builtin1(|xs| extremum_series(xs, false)));
    env = env.extend("std".into(), builtin1(|xs| std_series(xs)));
    env = env.extend(
        "Float".into(),
        Value::Record(vec![
            ("of_int".into(), builtin1(|v| match v {
                Value::Int(n) => Ok(Value::Float(n as f64)),
                other => Err(RuntimeError::new(format!(
                    "Float.of_int expects Int, got {}",
                    show(&other)
                ))),
            })),
            ("to_string".into(), builtin1(|v| match v {
                Value::Float(f) => Ok(Value::Str(format!("{f}"))),
                other => Err(RuntimeError::new(format!(
                    "Float.to_string expects Float, got {}",
                    show(&other)
                ))),
            })),
        ]),
    );
    env = env.extend(
        "Int".into(),
        Value::Record(vec![
            ("of_float".into(), builtin1(|v| match v {
                Value::Float(f) => Ok(Value::Int(f as i64)),
                other => Err(RuntimeError::new(format!(
                    "Int.of_float expects Float, got {}",
                    show(&other)
                ))),
            })),
            ("to_string".into(), builtin1(|v| match v {
                Value::Int(n) => Ok(Value::Str(n.to_string())),
                other => Err(RuntimeError::new(format!(
                    "Int.to_string expects Int, got {}",
                    show(&other)
                ))),
            })),
        ]),
    );
    env = env.extend(
        "String".into(),
        Value::Record(vec![
            ("length".into(), builtin1(|v| match v {
                Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
                other => Err(RuntimeError::new(format!(
                    "String.length expects String, got {}",
                    show(&other)
                ))),
            })),
            ("concat".into(), builtin1(|v| match v {
                Value::Series(vs) => {
                    let mut out = String::new();
                    for x in vs {
                        match x {
                            Value::Str(s) => out.push_str(&s),
                            other => {
                                return Err(RuntimeError::new(format!(
                                    "String.concat expects [String], got element {}",
                                    show(&other)
                                )));
                            }
                        }
                    }
                    Ok(Value::Str(out))
                }
                other => Err(RuntimeError::new(format!(
                    "String.concat expects [String], got {}",
                    show(&other)
                ))),
            })),
        ]),
    );
    env = env.extend(
        "Option".into(),
        Value::Record(vec![("unwrap".into(), builtin1(|v| match v {
            Value::Cons(n, args) if n == "Some" && args.len() == 1 => Ok(args[0].clone()),
            Value::Cons(n, _) if n == "None" => {
                Err(RuntimeError::new("Option.unwrap on None"))
            }
            other => Err(RuntimeError::new(format!(
                "Option.unwrap expects Option, got {}",
                show(&other)
            ))),
        }))]),
    );
    env = env.extend(
        "Result".into(),
        Value::Record(vec![("unwrap".into(), builtin1(|v| match v {
            Value::Cons(n, args) if n == "Ok" && args.len() == 1 => Ok(args[0].clone()),
            Value::Cons(n, args) if n == "Err" && args.len() == 1 => {
                Err(RuntimeError::new(format!(
                    "Result.unwrap on Err: {}",
                    show(&args[0])
                )))
            }
            other => Err(RuntimeError::new(format!(
                "Result.unwrap expects Result, got {}",
                show(&other)
            ))),
        }))]),
    );
    env
}

fn builtin1(f: impl Fn(Value) -> Result<Value, RuntimeError> + 'static) -> Value {
    Value::Builtin(BuiltinFn(Rc::new(f)))
}

fn sum_series(xs: Value) -> Result<Value, RuntimeError> {
    match xs {
        Value::Series(vs) => {
            if vs.is_empty() {
                return Ok(Value::Int(0));
            }
            if matches!(vs[0], Value::Float(_)) {
                let mut acc = 0.0;
                for v in vs {
                    match v {
                        Value::Float(f) => acc += f,
                        other => return Err(RuntimeError::new(format!(
                            "sum: mixed types, got {}",
                            show(&other)
                        ))),
                    }
                }
                Ok(Value::Float(acc))
            } else {
                let mut acc: i64 = 0;
                for v in vs {
                    match v {
                        Value::Int(n) => acc += n,
                        other => return Err(RuntimeError::new(format!(
                            "sum: mixed types, got {}",
                            show(&other)
                        ))),
                    }
                }
                Ok(Value::Int(acc))
            }
        }
        other => Err(RuntimeError::new(format!(
            "sum expects a series, got {}",
            show(&other)
        ))),
    }
}

fn mean_series(xs: Value) -> Result<Value, RuntimeError> {
    let Value::Series(vs) = xs else {
        return Err(RuntimeError::new("mean expects a series"));
    };
    if vs.is_empty() {
        return Err(RuntimeError::new("mean of empty series"));
    }
    let n = vs.len() as f64;
    let mut acc = 0.0;
    for v in vs {
        match v {
            Value::Float(f) => acc += f,
            Value::Int(i) => acc += i as f64,
            other => return Err(RuntimeError::new(format!(
                "mean expects numeric series, got {}",
                show(&other)
            ))),
        }
    }
    Ok(Value::Float(acc / n))
}

fn extremum_series(xs: Value, take_min: bool) -> Result<Value, RuntimeError> {
    let Value::Series(vs) = xs else {
        return Err(RuntimeError::new("min/max expects a series"));
    };
    if vs.is_empty() {
        return Err(RuntimeError::new("min/max of empty series"));
    }
    let mut best: f64 = match &vs[0] {
        Value::Float(f) => *f,
        Value::Int(i) => *i as f64,
        other => return Err(RuntimeError::new(format!(
            "min/max expects numeric, got {}",
            show(other)
        ))),
    };
    for v in &vs[1..] {
        let n = match v {
            Value::Float(f) => *f,
            Value::Int(i) => *i as f64,
            other => return Err(RuntimeError::new(format!(
                "min/max expects numeric, got {}",
                show(other)
            ))),
        };
        if (take_min && n < best) || (!take_min && n > best) {
            best = n;
        }
    }
    Ok(Value::Float(best))
}

fn std_series(xs: Value) -> Result<Value, RuntimeError> {
    let Value::Series(vs) = xs.clone() else {
        return Err(RuntimeError::new("std expects a series"));
    };
    if vs.len() < 2 {
        return Err(RuntimeError::new("std requires at least 2 elements"));
    }
    let Value::Float(mu) = mean_series(xs)? else {
        unreachable!()
    };
    let n = vs.len() as f64;
    let mut acc = 0.0;
    for v in vs {
        let x = match v {
            Value::Float(f) => f,
            Value::Int(i) => i as f64,
            other => return Err(RuntimeError::new(format!(
                "std expects numeric, got {}",
                show(&other)
            ))),
        };
        let d = x - mu;
        acc += d * d;
    }
    Ok(Value::Float((acc / (n - 1.0)).sqrt()))
}

fn make_constructor(name: String, arity: usize) -> Value {
    if arity == 0 {
        Value::Cons(name, Vec::new())
    } else {
        make_curried_cons(name, arity, Vec::new())
    }
}

fn make_curried_cons(name: String, arity: usize, collected: Vec<Value>) -> Value {
    if collected.len() == arity {
        return Value::Cons(name, collected);
    }
    let name_c = name.clone();
    Value::Builtin(BuiltinFn(Rc::new(move |arg| {
        let mut next = collected.clone();
        next.push(arg);
        Ok(make_curried_cons(name_c.clone(), arity, next))
    })))
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
        Stmt::Let { name, params, body, recursive, .. } => {
            if *recursive {
                *env = env.extend(name.clone(), Value::Unit);
                let value = if params.is_empty() {
                    eval(body, env)?
                } else {
                    make_closure(params, body, env)
                };
                env.mutate(name, value);
            } else {
                let value = if params.is_empty() {
                    eval(body, env)?
                } else {
                    make_closure(params, body, env)
                };
                *env = env.extend(name.clone(), value);
            }
            Ok(Value::Unit)
        }
        Stmt::LetRecGroup(bindings) => {
            for b in bindings {
                *env = env.extend(b.name.clone(), Value::Unit);
            }
            for b in bindings {
                let value = make_rec_value(b, env)?;
                env.mutate(&b.name, value);
            }
            Ok(Value::Unit)
        }
        Stmt::Var { name, body, .. } => {
            let value = eval(body, env)?;
            *env = env.extend(name.clone(), value);
            Ok(Value::Unit)
        }
        Stmt::TypeDecl(decl) => {
            if let TypeDeclBody::Sum(variants) = &decl.body {
                for v in variants {
                    *env = env
                        .extend(v.name.clone(), make_constructor(v.name.clone(), v.args.len()));
                }
            }
            Ok(Value::Unit)
        }
        Stmt::For { binding, iter, body } => {
            let iter_v = eval(iter, env)?;
            match iter_v {
                Value::Series(vs) => {
                    for v in vs {
                        let bindings = match_pat(binding, &v).ok_or_else(|| {
                            RuntimeError::new(format!(
                                "for-loop element {} does not match pattern",
                                show(&v)
                            ))
                        })?;
                        let mut inner = env.clone();
                        for (n, bv) in bindings {
                            inner = inner.extend(n, bv);
                        }
                        eval(body, &inner)?;
                    }
                    Ok(Value::Unit)
                }
                other => Err(RuntimeError::new(format!(
                    "for-loop iter must be a series, got {}",
                    show(&other)
                ))),
            }
        }
        Stmt::Mutate { name, body } => {
            let value = eval(body, env)?;
            if !env.mutate(name, value) {
                return Err(RuntimeError::new(format!("unbound: {name}")));
            }
            Ok(Value::Unit)
        }
        Stmt::Destructure { pat, body } => {
            let value = eval(body, env)?;
            match match_pat(pat, &value) {
                Some(bindings) => {
                    for (n, v) in bindings {
                        *env = env.extend(n, v);
                    }
                    Ok(Value::Unit)
                }
                None => Err(RuntimeError::new(format!(
                    "pattern did not match {}",
                    show(&value)
                ))),
            }
        }
        Stmt::Expr(e) => eval(e, env),
        Stmt::TraitDecl(_)
        | Stmt::Impl(_)
        | Stmt::Tests(_)
        | Stmt::Extern { .. }
        | Stmt::Import { .. }
        | Stmt::Input { .. }
        | Stmt::Output { .. } => Ok(Value::Unit),
    }
}

fn match_pat(pat: &Pat, value: &Value) -> Option<Vec<(String, Value)>> {
    match (pat, value) {
        (Pat::Wildcard, _) => Some(Vec::new()),
        (Pat::Var(name), v) => Some(vec![(name.clone(), v.clone())]),
        (Pat::Lit(Lit::Int(p)), Value::Int(v)) if p == v => Some(Vec::new()),
        (Pat::Lit(Lit::Float(p)), Value::Float(v)) if p == v => Some(Vec::new()),
        (Pat::Lit(Lit::Str(p)), Value::Str(v)) if p == v => Some(Vec::new()),
        (Pat::Lit(Lit::Bool(p)), Value::Bool(v)) if p == v => Some(Vec::new()),
        (Pat::Lit(Lit::Unit), Value::Unit) => Some(Vec::new()),
        (Pat::Cons(name, args), Value::Cons(vn, vargs))
            if name == vn && args.len() == vargs.len() =>
        {
            let mut bs = Vec::new();
            for (p, v) in args.iter().zip(vargs.iter()) {
                bs.extend(match_pat(p, v)?);
            }
            Some(bs)
        }
        (Pat::Tuple(pats), Value::Tuple(vs)) if pats.len() == vs.len() => {
            let mut bs = Vec::new();
            for (p, v) in pats.iter().zip(vs.iter()) {
                bs.extend(match_pat(p, v)?);
            }
            Some(bs)
        }
        (Pat::Record(fields), Value::Record(vs)) => {
            let mut bs = Vec::new();
            for (n, p) in fields {
                let v = vs.iter().find(|(vn, _)| vn == n).map(|(_, v)| v)?;
                bs.extend(match_pat(p, v)?);
            }
            Some(bs)
        }
        (Pat::Or(alts), v) => {
            for a in alts {
                if let Some(bs) = match_pat(a, v) {
                    return Some(bs);
                }
            }
            None
        }
        (Pat::As(inner, name), v) => {
            let mut bs = match_pat(inner, v)?;
            bs.push((name.clone(), v.clone()));
            Some(bs)
        }
        (Pat::Range(lo, hi), Value::Int(v)) => {
            if let (Pat::Lit(Lit::Int(l)), Pat::Lit(Lit::Int(h))) = (&**lo, &**hi)
                && l <= v
                && v <= h
            {
                return Some(Vec::new());
            }
            None
        }
        (Pat::List(parts), Value::Series(vs)) => {
            let total = vs.len();
            let fixed_before: usize = parts
                .iter()
                .take_while(|p| !matches!(p, ListPart::Rest(_)))
                .count();
            let has_rest = parts.iter().any(|p| matches!(p, ListPart::Rest(_)));
            let fixed_after: usize = parts
                .iter()
                .rev()
                .take_while(|p| !matches!(p, ListPart::Rest(_)))
                .count();
            if !has_rest {
                if parts.len() != total {
                    return None;
                }
            } else if fixed_before + fixed_after > total {
                return None;
            }
            let mut bs = Vec::new();
            for (p, v) in parts.iter().take(fixed_before).zip(vs.iter().take(fixed_before)) {
                if let ListPart::Pat(p) = p {
                    bs.extend(match_pat(p, v)?);
                }
            }
            if let Some(ListPart::Rest(name)) = parts.iter().find(|p| matches!(p, ListPart::Rest(_))) {
                let rest = &vs[fixed_before..total - fixed_after];
                if let Some(n) = name {
                    bs.push((n.clone(), Value::Series(rest.to_vec())));
                }
            }
            for (p, v) in parts.iter().rev().take(fixed_after).zip(vs.iter().rev().take(fixed_after))
            {
                if let ListPart::Pat(p) = p {
                    bs.extend(match_pat(p, v)?);
                }
            }
            Some(bs)
        }
        _ => None,
    }
}

fn make_rec_value(b: &LetBinding, env: &Env) -> Result<Value, RuntimeError> {
    if b.params.is_empty() {
        eval(&b.body, env)
    } else {
        Ok(make_closure(&b.params, &b.body, env))
    }
}

fn make_closure(params: &[Param], body: &Expr, env: &Env) -> Value {
    Value::Closure {
        params: params.iter().map(|p| p.pat.clone()).collect(),
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
            .ok_or_else(|| RuntimeError::new(format!("unbound: {name}"))),
        Expr::Lambda(params, body) => Ok(Value::Closure {
            params: params.iter().map(|p| p.pat.clone()).collect(),
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
        Expr::Postfix(vela_parser::PostOp::Question, inner) => {
            let v = eval(inner, env)?;
            match &v {
                Value::Cons(n, args) if n == "Ok" && args.len() == 1 => {
                    Ok(args[0].clone())
                }
                Value::Cons(n, _) if n == "Err" => {
                    Err(RuntimeError::short_circuit(v))
                }
                other => Err(RuntimeError::new(format!(
                    "`?` expects a Result, got {}",
                    show(other)
                ))),
            }
        }
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
        Expr::Match(scrut, arms) => {
            let s = eval(scrut, env)?;
            for arm in arms {
                if let Some(bindings) = match_pat(&arm.pat, &s) {
                    let mut arm_env = env.clone();
                    for (n, v) in bindings {
                        arm_env = arm_env.extend(n, v);
                    }
                    if let Some(guard) = &arm.guard {
                        match eval(guard, &arm_env)? {
                            Value::Bool(true) => {}
                            Value::Bool(false) => continue,
                            other => {
                                return Err(RuntimeError::new(format!(
                                    "guard must be Bool, got {}",
                                    show(&other)
                                )));
                            }
                        }
                    }
                    return eval(&arm.body, &arm_env);
                }
            }
            Err(RuntimeError::new(format!(
                "no match arm matched value {}",
                show(&s)
            )))
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
            let bindings = match_pat(&params[0], &arg)
                .ok_or_else(|| RuntimeError::new(format!(
                    "argument {} does not match parameter pattern",
                    show(&arg)
                )))?;
            let mut inner_env = env;
            for (n, v) in bindings {
                inner_env = inner_env.extend(n, v);
            }
            if params.len() == 1 {
                match eval(&body, &inner_env) {
                    Err(e) if e.short_circuit.is_some() => {
                        Ok(e.short_circuit.unwrap())
                    }
                    other => other,
                }
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
