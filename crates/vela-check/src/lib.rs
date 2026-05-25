//! Type checking and inference for the Vela language.

use std::collections::HashMap;
use vela_parser::{BinOp, Expr, Lit, Pat, PostOp, Stmt, UnOp, parse_expr, parse_program};

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    UInt,
    BigInt,
    Float,
    Decimal,
    Bool,
    String,
    Symbol,
    Unit,
    Var(u32),
    Fn(Box<Type>, Box<Type>),
    Series(Box<Type>),
    Tuple(Vec<Type>),
    Record(Vec<(String, Type)>),
    DataFrame,
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
}

impl Type {
    fn show(&self) -> String {
        match self {
            Type::Int => "Int".into(),
            Type::UInt => "UInt".into(),
            Type::BigInt => "BigInt".into(),
            Type::Float => "Float".into(),
            Type::Decimal => "Decimal".into(),
            Type::Bool => "Bool".into(),
            Type::String => "String".into(),
            Type::Symbol => "Symbol".into(),
            Type::Unit => "()".into(),
            Type::Var(n) => format!("'t{n}"),
            Type::Fn(a, b) => format!("({} -> {})", a.show(), b.show()),
            Type::Series(t) => format!("[{}]", t.show()),
            Type::Tuple(ts) => {
                let parts: Vec<String> = ts.iter().map(|t| t.show()).collect();
                format!("({})", parts.join(", "))
            }
            Type::Record(fields) => {
                let parts: Vec<String> =
                    fields.iter().map(|(n, t)| format!("{n}: {}", t.show())).collect();
                format!("{{ {} }}", parts.join(", "))
            }
            Type::DataFrame => "DataFrame".into(),
            Type::Option(t) => format!("Option[{}]", t.show()),
            Type::Result(a, e) => format!("Result[{}, {}]", a.show(), e.show()),
        }
    }

    fn is_numeric(&self) -> bool {
        matches!(
            self,
            Type::Int | Type::UInt | Type::BigInt | Type::Float | Type::Decimal,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub message: String,
}

impl TypeError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Scheme {
    vars: Vec<u32>,
    ty: Type,
}

impl Scheme {
    fn mono(ty: Type) -> Self {
        Self { vars: Vec::new(), ty }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Env {
    bindings: HashMap<String, Scheme>,
}

impl Env {
    pub fn new() -> Self {
        Self::default()
    }

    fn extend(&self, name: String, scheme: Scheme) -> Env {
        let mut bindings = self.bindings.clone();
        bindings.insert(name, scheme);
        Env { bindings }
    }

    fn lookup(&self, name: &str) -> Option<&Scheme> {
        self.bindings.get(name)
    }
}

#[derive(Debug, Default)]
struct Ctx {
    subst: HashMap<u32, Type>,
    fresh: u32,
}

impl Ctx {
    fn fresh_var(&mut self) -> Type {
        let n = self.fresh;
        self.fresh += 1;
        Type::Var(n)
    }

    fn resolve(&self, t: &Type) -> Type {
        match t {
            Type::Var(n) => match self.subst.get(n) {
                Some(t2) => self.resolve(t2),
                None => Type::Var(*n),
            },
            Type::Fn(a, b) => {
                Type::Fn(Box::new(self.resolve(a)), Box::new(self.resolve(b)))
            }
            Type::Series(t) => Type::Series(Box::new(self.resolve(t))),
            Type::Tuple(ts) => Type::Tuple(ts.iter().map(|t| self.resolve(t)).collect()),
            Type::Record(fields) => Type::Record(
                fields.iter().map(|(n, t)| (n.clone(), self.resolve(t))).collect(),
            ),
            Type::Option(t) => Type::Option(Box::new(self.resolve(t))),
            Type::Result(a, e) => {
                Type::Result(Box::new(self.resolve(a)), Box::new(self.resolve(e)))
            }
            other => other.clone(),
        }
    }

    fn instantiate(&mut self, scheme: &Scheme) -> Type {
        let mut subst: HashMap<u32, Type> = HashMap::new();
        for &v in &scheme.vars {
            subst.insert(v, self.fresh_var());
        }
        apply_subst(&scheme.ty, &subst)
    }

    fn generalize(&self, env: &Env, ty: &Type) -> Scheme {
        let resolved = self.resolve(ty);
        let mut ty_ftv = std::collections::BTreeSet::new();
        collect_ftv(&resolved, self, &mut ty_ftv);
        let mut env_ftv = std::collections::BTreeSet::new();
        for scheme in env.bindings.values() {
            let resolved_scheme_ty = self.resolve(&scheme.ty);
            let mut s_ftv = std::collections::BTreeSet::new();
            collect_ftv(&resolved_scheme_ty, self, &mut s_ftv);
            for v in scheme.vars.iter() {
                s_ftv.remove(v);
            }
            env_ftv.extend(s_ftv);
        }
        let vars: Vec<u32> = ty_ftv.difference(&env_ftv).copied().collect();
        Scheme { vars, ty: resolved }
    }

    fn occurs(&self, n: u32, t: &Type) -> bool {
        match self.resolve(t) {
            Type::Var(m) => m == n,
            Type::Fn(a, b) => self.occurs(n, &a) || self.occurs(n, &b),
            Type::Series(t) => self.occurs(n, &t),
            Type::Tuple(ts) => ts.iter().any(|t| self.occurs(n, t)),
            Type::Record(fields) => fields.iter().any(|(_, t)| self.occurs(n, t)),
            Type::Option(t) => self.occurs(n, &t),
            Type::Result(a, e) => self.occurs(n, &a) || self.occurs(n, &e),
            _ => false,
        }
    }

    fn unify(&mut self, a: &Type, b: &Type) -> Result<(), TypeError> {
        let a = self.resolve(a);
        let b = self.resolve(b);
        if a == b {
            return Ok(());
        }
        match (a, b) {
            (Type::Var(n), t) | (t, Type::Var(n)) => {
                if self.occurs(n, &t) {
                    return Err(TypeError::new(format!(
                        "infinite type: 't{n} = {}",
                        t.show()
                    )));
                }
                self.subst.insert(n, t);
                Ok(())
            }
            (Type::Fn(a1, b1), Type::Fn(a2, b2)) => {
                self.unify(&a1, &a2)?;
                self.unify(&b1, &b2)
            }
            (Type::Series(a), Type::Series(b)) => self.unify(&a, &b),
            (Type::Tuple(a), Type::Tuple(b)) => {
                if a.len() != b.len() {
                    return Err(TypeError::new(format!(
                        "tuple arity mismatch: {} vs {}",
                        a.len(),
                        b.len()
                    )));
                }
                for (x, y) in a.iter().zip(b.iter()) {
                    self.unify(x, y)?;
                }
                Ok(())
            }
            (Type::Option(a), Type::Option(b)) => self.unify(&a, &b),
            (Type::Result(a1, e1), Type::Result(a2, e2)) => {
                self.unify(&a1, &a2)?;
                self.unify(&e1, &e2)
            }
            (Type::Record(a), Type::Record(b)) => {
                if a.len() != b.len() {
                    return Err(TypeError::new(format!(
                        "record field count mismatch: {} vs {}",
                        a.len(),
                        b.len()
                    )));
                }
                for ((n1, t1), (n2, t2)) in a.iter().zip(b.iter()) {
                    if n1 != n2 {
                        return Err(TypeError::new(format!(
                            "record field name mismatch: `{n1}` vs `{n2}`"
                        )));
                    }
                    self.unify(t1, t2)?;
                }
                Ok(())
            }
            (a, b) => Err(TypeError::new(format!(
                "cannot unify {} with {}",
                a.show(),
                b.show()
            ))),
        }
    }
}

pub fn check_expr(src: &str) -> Result<Type, TypeError> {
    let expr = parse_expr(src).map_err(|e| TypeError::new(format!("parse error: {}", e.message)))?;
    let mut ctx = Ctx::default();
    let env = prelude();
    let t = infer(&expr, &env, &mut ctx)?;
    Ok(ctx.resolve(&t))
}

pub fn check_program(src: &str) -> Result<Type, TypeError> {
    let program = parse_program(src)
        .map_err(|e| TypeError::new(format!("parse error: {}", e.message)))?;
    let mut ctx = Ctx::default();
    let mut env = prelude();
    let mut last = Type::Unit;
    for stmt in &program.stmts {
        last = check_stmt(stmt, &mut env, &mut ctx)?;
    }
    Ok(ctx.resolve(&last))
}

fn prelude() -> Env {
    let mut env = Env::new();
    // None : forall a. Option a
    env = env.extend(
        "None".into(),
        Scheme { vars: vec![0], ty: Type::Option(Box::new(Type::Var(0))) },
    );
    // Some : forall a. a -> Option a
    env = env.extend(
        "Some".into(),
        Scheme {
            vars: vec![0],
            ty: Type::Fn(
                Box::new(Type::Var(0)),
                Box::new(Type::Option(Box::new(Type::Var(0)))),
            ),
        },
    );
    // Ok : forall a e. a -> Result a e
    env = env.extend(
        "Ok".into(),
        Scheme {
            vars: vec![0, 1],
            ty: Type::Fn(
                Box::new(Type::Var(0)),
                Box::new(Type::Result(
                    Box::new(Type::Var(0)),
                    Box::new(Type::Var(1)),
                )),
            ),
        },
    );
    // Err : forall a e. e -> Result a e
    env = env.extend(
        "Err".into(),
        Scheme {
            vars: vec![0, 1],
            ty: Type::Fn(
                Box::new(Type::Var(1)),
                Box::new(Type::Result(
                    Box::new(Type::Var(0)),
                    Box::new(Type::Var(1)),
                )),
            ),
        },
    );
    env
}

fn check_stmt(stmt: &Stmt, env: &mut Env, ctx: &mut Ctx) -> Result<Type, TypeError> {
    match stmt {
        Stmt::Let { name, params, return_ty, body } => {
            let mut translator = TyTranslator::new();
            let mut inner_env = env.clone();
            let mut param_types = Vec::with_capacity(params.len());
            for p in params {
                let pt = match &p.ty {
                    Some(ty) => translator.translate(ty, ctx)?,
                    None => ctx.fresh_var(),
                };
                inner_env = inner_env.extend(p.name.clone(), Scheme::mono(pt.clone()));
                param_types.push(pt);
            }
            let body_ty = infer(body, &inner_env, ctx)?;
            if let Some(rt) = return_ty {
                let rt_translated = translator.translate(rt, ctx)?;
                ctx.unify(&body_ty, &rt_translated)?;
            }
            let ty = param_types.into_iter().rev().fold(body_ty, |acc, pt| {
                Type::Fn(Box::new(pt), Box::new(acc))
            });
            let scheme = ctx.generalize(env, &ty);
            *env = env.extend(name.clone(), scheme);
            Ok(Type::Unit)
        }
        Stmt::Var { name, ty, body } => {
            let body_ty = infer(body, env, ctx)?;
            if let Some(annotation) = ty {
                let mut translator = TyTranslator::new();
                let annotation_ty = translator.translate(annotation, ctx)?;
                ctx.unify(&body_ty, &annotation_ty)?;
            }
            let resolved = ctx.resolve(&body_ty);
            *env = env.extend(name.clone(), Scheme::mono(resolved));
            Ok(Type::Unit)
        }
        Stmt::Expr(e) => infer(e, env, ctx),
        other => Err(TypeError::new(format!("cannot yet check {other:?}"))),
    }
}

fn lambda_type(
    params: &[vela_parser::Param],
    body: &Expr,
    env: &Env,
    ctx: &mut Ctx,
) -> Result<Type, TypeError> {
    let mut env = env.clone();
    let mut param_types = Vec::with_capacity(params.len());
    for p in params {
        let pt = ctx.fresh_var();
        env = env.extend(p.name.clone(), Scheme::mono(pt.clone()));
        param_types.push(pt);
    }
    let body_ty = infer(body, &env, ctx)?;
    Ok(param_types
        .into_iter()
        .rev()
        .fold(body_ty, |acc, pt| Type::Fn(Box::new(pt), Box::new(acc))))
}

fn infer(expr: &Expr, env: &Env, ctx: &mut Ctx) -> Result<Type, TypeError> {
    match expr {
        Expr::Lit(Lit::Int(_)) => Ok(Type::Int),
        Expr::Lit(Lit::Float(_)) => Ok(Type::Float),
        Expr::Lit(Lit::Str(_)) => Ok(Type::String),
        Expr::Lit(Lit::Bool(_)) => Ok(Type::Bool),
        Expr::Lit(Lit::Unit) => Ok(Type::Unit),
        Expr::Var(name) => {
            let scheme = env
                .lookup(name)
                .cloned()
                .ok_or_else(|| TypeError::new(format!("unbound name: {name}")))?;
            Ok(ctx.instantiate(&scheme))
        }
        Expr::UnaryOp(op, inner) => infer_unary(*op, inner, env, ctx),
        Expr::Postfix(PostOp::Question, inner) => {
            let t = infer(inner, env, ctx)?;
            let a = ctx.fresh_var();
            let e = ctx.fresh_var();
            let expected = Type::Result(Box::new(a.clone()), Box::new(e));
            ctx.unify(&t, &expected)?;
            Ok(ctx.resolve(&a))
        }
        Expr::BinOp(op, lhs, rhs) => infer_binary(*op, lhs, rhs, env, ctx),
        Expr::Lambda(params, body) => {
            let params: Vec<vela_parser::Param> = params
                .iter()
                .map(|n| vela_parser::Param { name: n.clone(), ty: None })
                .collect();
            lambda_type(&params, body, env, ctx)
        }
        Expr::Block { stmts, trailing } => {
            let mut block_env = env.clone();
            for s in stmts {
                check_stmt(s, &mut block_env, ctx)?;
            }
            match trailing {
                Some(e) => infer(e, &block_env, ctx),
                None => Ok(Type::Unit),
            }
        }
        Expr::App(f, arg) => {
            let f_ty = infer(f, env, ctx)?;
            let arg_ty = infer(arg, env, ctx)?;
            let result = ctx.fresh_var();
            let expected = Type::Fn(Box::new(arg_ty), Box::new(result.clone()));
            ctx.unify(&f_ty, &expected)?;
            Ok(ctx.resolve(&result))
        }
        Expr::If(cond, then_b, else_b) => {
            let cond_ty = infer(cond, env, ctx)?;
            ctx.unify(&cond_ty, &Type::Bool)?;
            let then_ty = infer(then_b, env, ctx)?;
            let else_ty = infer(else_b, env, ctx)?;
            ctx.unify(&then_ty, &else_ty)?;
            Ok(ctx.resolve(&then_ty))
        }
        Expr::Sym(_) => Ok(Type::Symbol),
        Expr::Series(elems) => {
            let inner = ctx.fresh_var();
            for e in elems {
                let t = infer(e, env, ctx)?;
                ctx.unify(&inner, &t)?;
            }
            Ok(Type::Series(Box::new(ctx.resolve(&inner))))
        }
        Expr::Tuple(elems) => {
            let mut ts = Vec::with_capacity(elems.len());
            for e in elems {
                ts.push(infer(e, env, ctx)?);
            }
            Ok(Type::Tuple(ts.into_iter().map(|t| ctx.resolve(&t)).collect()))
        }
        Expr::Record(fields) => {
            let mut fts = Vec::with_capacity(fields.len());
            for (n, e) in fields {
                let t = infer(e, env, ctx)?;
                fts.push((n.clone(), ctx.resolve(&t)));
            }
            Ok(Type::Record(fts))
        }
        Expr::RecordUpdate(base, updates) => {
            let base_ty = infer(base, env, ctx)?;
            let base_ty = ctx.resolve(&base_ty);
            let Type::Record(mut fields) = base_ty else {
                return Err(TypeError::new(format!(
                    "record update requires a record, got {}",
                    base_ty.show()
                )));
            };
            for (n, e) in updates {
                let t = infer(e, env, ctx)?;
                let t = ctx.resolve(&t);
                let Some((_, existing)) = fields.iter_mut().find(|(name, _)| name == n) else {
                    return Err(TypeError::new(format!(
                        "record has no field `{n}`"
                    )));
                };
                ctx.unify(existing, &t)?;
                *existing = ctx.resolve(&t);
            }
            Ok(Type::Record(fields))
        }
        Expr::Field(target, name) => {
            let t = infer(target, env, ctx)?;
            let t = ctx.resolve(&t);
            let Type::Record(fields) = &t else {
                return Err(TypeError::new(format!(
                    "field access requires a record, got {}",
                    t.show()
                )));
            };
            fields
                .iter()
                .find(|(n, _)| n == name)
                .map(|(_, ty)| ty.clone())
                .ok_or_else(|| TypeError::new(format!("record has no field `{name}`")))
        }
        Expr::Match(scrut, arms) => {
            let s_ty = infer(scrut, env, ctx)?;
            let result_ty = ctx.fresh_var();
            for arm in arms {
                let (pat_ty, bindings) = infer_pat(&arm.pat, ctx)?;
                ctx.unify(&s_ty, &pat_ty)?;
                let mut arm_env = env.clone();
                for (n, t) in bindings {
                    arm_env = arm_env.extend(n, Scheme::mono(t));
                }
                if let Some(g) = &arm.guard {
                    let gt = infer(g, &arm_env, ctx)?;
                    ctx.unify(&gt, &Type::Bool)?;
                }
                let body_ty = infer(&arm.body, &arm_env, ctx)?;
                ctx.unify(&result_ty, &body_ty)?;
            }
            Ok(ctx.resolve(&result_ty))
        }
        other => Err(TypeError::new(format!("cannot yet infer type of {other:?}"))),
    }
}

struct TyTranslator {
    named_vars: HashMap<String, Type>,
}

impl TyTranslator {
    fn new() -> Self {
        Self { named_vars: HashMap::new() }
    }

    fn translate(
        &mut self,
        ty: &vela_parser::Ty,
        ctx: &mut Ctx,
    ) -> Result<Type, TypeError> {
        match ty {
            vela_parser::Ty::Unit => Ok(Type::Unit),
            vela_parser::Ty::Con(name) => match name.as_str() {
                "Int" => Ok(Type::Int),
                "UInt" => Ok(Type::UInt),
                "BigInt" => Ok(Type::BigInt),
                "Float" => Ok(Type::Float),
                "Decimal" => Ok(Type::Decimal),
                "Bool" => Ok(Type::Bool),
                "String" => Ok(Type::String),
                "Symbol" => Ok(Type::Symbol),
                "DataFrame" => Ok(Type::DataFrame),
                other => Err(TypeError::new(format!("unknown type: {other}"))),
            },
            vela_parser::Ty::Var(name) => {
                if let Some(t) = self.named_vars.get(name) {
                    Ok(t.clone())
                } else {
                    let v = ctx.fresh_var();
                    self.named_vars.insert(name.clone(), v.clone());
                    Ok(v)
                }
            }
            vela_parser::Ty::Series(t) => {
                Ok(Type::Series(Box::new(self.translate(t, ctx)?)))
            }
            vela_parser::Ty::Tuple(ts) => {
                let mut translated = Vec::with_capacity(ts.len());
                for t in ts {
                    translated.push(self.translate(t, ctx)?);
                }
                Ok(Type::Tuple(translated))
            }
            vela_parser::Ty::Record(fields) => {
                let mut translated = Vec::with_capacity(fields.len());
                for (n, t) in fields {
                    translated.push((n.clone(), self.translate(t, ctx)?));
                }
                Ok(Type::Record(translated))
            }
            other => Err(TypeError::new(format!("cannot translate type: {other:?}"))),
        }
    }
}

fn apply_subst(ty: &Type, subst: &HashMap<u32, Type>) -> Type {
    match ty {
        Type::Var(n) => subst.get(n).cloned().unwrap_or(Type::Var(*n)),
        Type::Fn(a, b) => {
            Type::Fn(Box::new(apply_subst(a, subst)), Box::new(apply_subst(b, subst)))
        }
        Type::Series(t) => Type::Series(Box::new(apply_subst(t, subst))),
        Type::Tuple(ts) => Type::Tuple(ts.iter().map(|t| apply_subst(t, subst)).collect()),
        Type::Record(fs) => Type::Record(
            fs.iter().map(|(n, t)| (n.clone(), apply_subst(t, subst))).collect(),
        ),
        Type::Option(t) => Type::Option(Box::new(apply_subst(t, subst))),
        Type::Result(a, e) => {
            Type::Result(Box::new(apply_subst(a, subst)), Box::new(apply_subst(e, subst)))
        }
        other => other.clone(),
    }
}

fn collect_ftv(ty: &Type, ctx: &Ctx, out: &mut std::collections::BTreeSet<u32>) {
    match ctx.resolve(ty) {
        Type::Var(n) => {
            out.insert(n);
        }
        Type::Fn(a, b) => {
            collect_ftv(&a, ctx, out);
            collect_ftv(&b, ctx, out);
        }
        Type::Series(t) => collect_ftv(&t, ctx, out),
        Type::Tuple(ts) => {
            for t in &ts {
                collect_ftv(t, ctx, out);
            }
        }
        Type::Record(fs) => {
            for (_, t) in &fs {
                collect_ftv(t, ctx, out);
            }
        }
        Type::Option(t) => collect_ftv(&t, ctx, out),
        Type::Result(a, e) => {
            collect_ftv(&a, ctx, out);
            collect_ftv(&e, ctx, out);
        }
        _ => {}
    }
}

fn infer_pat(pat: &Pat, ctx: &mut Ctx) -> Result<(Type, Vec<(String, Type)>), TypeError> {
    match pat {
        Pat::Wildcard => Ok((ctx.fresh_var(), Vec::new())),
        Pat::Var(name) => {
            let t = ctx.fresh_var();
            Ok((t.clone(), vec![(name.clone(), t)]))
        }
        Pat::Lit(Lit::Int(_)) => Ok((Type::Int, Vec::new())),
        Pat::Lit(Lit::Float(_)) => Ok((Type::Float, Vec::new())),
        Pat::Lit(Lit::Str(_)) => Ok((Type::String, Vec::new())),
        Pat::Lit(Lit::Bool(_)) => Ok((Type::Bool, Vec::new())),
        Pat::Lit(Lit::Unit) => Ok((Type::Unit, Vec::new())),
        Pat::As(inner, name) => {
            let (t, mut bs) = infer_pat(inner, ctx)?;
            bs.push((name.clone(), t.clone()));
            Ok((t, bs))
        }
        Pat::Or(alts) => {
            if alts.is_empty() {
                return Err(TypeError::new("empty or-pattern"));
            }
            let (t, bs) = infer_pat(&alts[0], ctx)?;
            for a in &alts[1..] {
                let (t2, _) = infer_pat(a, ctx)?;
                ctx.unify(&t, &t2)?;
            }
            Ok((t, bs))
        }
        other => Err(TypeError::new(format!("cannot yet type pattern: {other:?}"))),
    }
}

fn infer_unary(op: UnOp, inner: &Expr, env: &Env, ctx: &mut Ctx) -> Result<Type, TypeError> {
    let t = infer(inner, env, ctx)?;
    let t = ctx.resolve(&t);
    match op {
        UnOp::Neg => {
            if t.is_numeric() {
                Ok(t)
            } else {
                Err(TypeError::new(format!("cannot negate {}", t.show())))
            }
        }
        UnOp::Not => {
            ctx.unify(&t, &Type::Bool)?;
            Ok(Type::Bool)
        }
    }
}

fn infer_binary(
    op: BinOp,
    lhs: &Expr,
    rhs: &Expr,
    env: &Env,
    ctx: &mut Ctx,
) -> Result<Type, TypeError> {
    let l = infer(lhs, env, ctx)?;
    let r = infer(rhs, env, ctx)?;
    let l = ctx.resolve(&l);
    let r = ctx.resolve(&r);
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::Pow => {
            ctx.unify(&l, &r)?;
            let resolved = ctx.resolve(&l);
            match resolved {
                Type::Var(_) => {
                    ctx.unify(&l, &Type::Int)?;
                    Ok(Type::Int)
                }
                t if t.is_numeric() => Ok(t),
                t => Err(TypeError::new(format!(
                    "arithmetic requires numeric, got {}",
                    t.show()
                ))),
            }
        }
        BinOp::Eq | BinOp::NotEq => {
            ctx.unify(&l, &r)?;
            Ok(Type::Bool)
        }
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            ctx.unify(&l, &r)?;
            Ok(Type::Bool)
        }
        BinOp::And | BinOp::Or => {
            ctx.unify(&l, &Type::Bool)?;
            ctx.unify(&r, &Type::Bool)?;
            Ok(Type::Bool)
        }
        BinOp::Concat => {
            ctx.unify(&l, &r)?;
            let resolved = ctx.resolve(&l);
            match resolved {
                Type::String | Type::Series(_) => Ok(resolved),
                Type::Var(_) => Err(TypeError::new(
                    "`++` operands are ambiguous; annotate them",
                )),
                other => Err(TypeError::new(format!(
                    "`++` requires String or Series, got {}",
                    other.show()
                ))),
            }
        }
        BinOp::Pipe => {
            let result = ctx.fresh_var();
            let expected = Type::Fn(Box::new(l), Box::new(result.clone()));
            ctx.unify(&r, &expected)?;
            Ok(ctx.resolve(&result))
        }
        BinOp::Tilde => Err(TypeError::new(
            "`~` (formula) typing requires DataFrame context; not yet implemented",
        )),
    }
}
