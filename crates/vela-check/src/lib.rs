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
}

impl Type {
    fn name(&self) -> &'static str {
        match self {
            Type::Int => "Int",
            Type::UInt => "UInt",
            Type::BigInt => "BigInt",
            Type::Float => "Float",
            Type::Decimal => "Decimal",
            Type::Bool => "Bool",
            Type::String => "String",
            Type::Symbol => "Symbol",
            Type::Unit => "()",
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

pub fn check_expr(src: &str) -> Result<Type, TypeError> {
    let expr = parse_expr(src).map_err(|e| TypeError::new(format!("parse error: {}", e.message)))?;
    infer(&expr, &Env::new())
}

pub fn check_program(src: &str) -> Result<Type, TypeError> {
    let program = parse_program(src)
        .map_err(|e| TypeError::new(format!("parse error: {}", e.message)))?;
    let mut env = Env::new();
    let mut last = Type::Unit;
    for stmt in &program.stmts {
        last = check_stmt(stmt, &mut env)?;
    }
    Ok(last)
}

fn check_stmt(stmt: &Stmt, env: &mut Env) -> Result<Type, TypeError> {
    match stmt {
        Stmt::Let { name, params, body, .. } if params.is_empty() => {
            let ty = infer(body, env)?;
            *env = env.extend(name.clone(), ty);
            Ok(Type::Unit)
        }
        Stmt::Expr(e) => infer(e, env),
        other => Err(TypeError::new(format!("cannot yet check {other:?}"))),
    }
}

fn infer(expr: &Expr, env: &Env) -> Result<Type, TypeError> {
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
        Expr::UnaryOp(op, inner) => infer_unary(*op, inner, env),
        Expr::BinOp(op, lhs, rhs) => infer_binary(*op, lhs, rhs, env),
        other => Err(TypeError::new(format!("cannot yet infer type of {other:?}"))),
    }
}

fn infer_unary(op: UnOp, inner: &Expr, env: &Env) -> Result<Type, TypeError> {
    let t = infer(inner, env)?;
    match op {
        UnOp::Neg => {
            if t.is_numeric() {
                Ok(t)
            } else {
                Err(TypeError::new(format!("cannot negate {}", t.name())))
            }
        }
        UnOp::Not => {
            if t == Type::Bool {
                Ok(Type::Bool)
            } else {
                Err(TypeError::new(format!("`not` requires Bool, got {}", t.name())))
            }
        }
    }
}

fn infer_binary(op: BinOp, lhs: &Expr, rhs: &Expr, env: &Env) -> Result<Type, TypeError> {
    let l = infer(lhs, env)?;
    let r = infer(rhs, env)?;
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod | BinOp::Pow => {
            if !l.is_numeric() {
                return Err(TypeError::new(format!(
                    "arithmetic requires numeric, got {}",
                    l.name()
                )));
            }
            if l != r {
                return Err(TypeError::new(format!(
                    "arithmetic operands must match: {} vs {}",
                    l.name(),
                    r.name()
                )));
            }
            Ok(l)
        }
        BinOp::Eq | BinOp::NotEq => {
            if l != r {
                return Err(TypeError::new(format!(
                    "equality operands must match: {} vs {}",
                    l.name(),
                    r.name()
                )));
            }
            Ok(Type::Bool)
        }
        BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
            if l != r {
                return Err(TypeError::new(format!(
                    "comparison operands must match: {} vs {}",
                    l.name(),
                    r.name()
                )));
            }
            Ok(Type::Bool)
        }
        BinOp::And | BinOp::Or => {
            if l != Type::Bool || r != Type::Bool {
                return Err(TypeError::new(format!(
                    "`{}` requires Bool, got {} and {}",
                    match op {
                        BinOp::And => "and",
                        BinOp::Or => "or",
                        _ => unreachable!(),
                    },
                    l.name(),
                    r.name()
                )));
            }
            Ok(Type::Bool)
        }
        BinOp::Concat => {
            if l != r {
                return Err(TypeError::new(format!(
                    "`++` operands must match: {} vs {}",
                    l.name(),
                    r.name()
                )));
            }
            if l != Type::String {
                return Err(TypeError::new(format!(
                    "`++` not yet supported for {}",
                    l.name()
                )));
            }
            Ok(Type::String)
        }
        BinOp::Pipe | BinOp::Tilde => Err(TypeError::new(format!(
            "`{op:?}` typing not yet implemented"
        ))),
    }
}
