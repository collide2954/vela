//! Type checking and inference for the Vela language.

use std::collections::HashMap;
use vela_parser::{BinOp, Expr, Lit, Stmt, UnOp, parse_expr, parse_program};

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

#[derive(Debug, Default, Clone)]
pub struct Env {
    bindings: HashMap<String, Type>,
}

impl Env {
    pub fn new() -> Self {
        Self::default()
    }

    fn extend(&self, name: String, ty: Type) -> Env {
        let mut bindings = self.bindings.clone();
        bindings.insert(name, ty);
        Env { bindings }
    }

    fn lookup(&self, name: &str) -> Option<&Type> {
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
            other => other.clone(),
        }
    }

    fn occurs(&self, n: u32, t: &Type) -> bool {
        match self.resolve(t) {
            Type::Var(m) => m == n,
            Type::Fn(a, b) => self.occurs(n, &a) || self.occurs(n, &b),
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
    let env = Env::new();
    let t = infer(&expr, &env, &mut ctx)?;
    Ok(ctx.resolve(&t))
}

pub fn check_program(src: &str) -> Result<Type, TypeError> {
    let program = parse_program(src)
        .map_err(|e| TypeError::new(format!("parse error: {}", e.message)))?;
    let mut ctx = Ctx::default();
    let mut env = Env::new();
    let mut last = Type::Unit;
    for stmt in &program.stmts {
        last = check_stmt(stmt, &mut env, &mut ctx)?;
    }
    Ok(ctx.resolve(&last))
}

fn check_stmt(stmt: &Stmt, env: &mut Env, ctx: &mut Ctx) -> Result<Type, TypeError> {
    match stmt {
        Stmt::Let { name, params, body, .. } if params.is_empty() => {
            let ty = infer(body, env, ctx)?;
            let ty = ctx.resolve(&ty);
            *env = env.extend(name.clone(), ty);
            Ok(Type::Unit)
        }
        Stmt::Let { name, params, body, .. } => {
            let lambda = lambda_type(params, body, env, ctx)?;
            *env = env.extend(name.clone(), ctx.resolve(&lambda));
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
        env = env.extend(p.name.clone(), pt.clone());
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
        Expr::Var(name) => env
            .lookup(name)
            .cloned()
            .ok_or_else(|| TypeError::new(format!("unbound name: {name}"))),
        Expr::UnaryOp(op, inner) => infer_unary(*op, inner, env, ctx),
        Expr::BinOp(op, lhs, rhs) => infer_binary(*op, lhs, rhs, env, ctx),
        Expr::Lambda(params, body) => {
            let params: Vec<vela_parser::Param> = params
                .iter()
                .map(|n| vela_parser::Param { name: n.clone(), ty: None })
                .collect();
            lambda_type(&params, body, env, ctx)
        }
        Expr::App(f, arg) => {
            let f_ty = infer(f, env, ctx)?;
            let arg_ty = infer(arg, env, ctx)?;
            let result = ctx.fresh_var();
            let expected = Type::Fn(Box::new(arg_ty), Box::new(result.clone()));
            ctx.unify(&f_ty, &expected)?;
            Ok(ctx.resolve(&result))
        }
        other => Err(TypeError::new(format!("cannot yet infer type of {other:?}"))),
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
            if resolved != Type::String {
                return Err(TypeError::new(format!(
                    "`++` not yet supported for {}",
                    resolved.show()
                )));
            }
            Ok(Type::String)
        }
        BinOp::Pipe | BinOp::Tilde => Err(TypeError::new(format!(
            "`{op:?}` typing not yet implemented"
        ))),
    }
}
