//! Type checking and inference for the Vela language.

use vela_parser::{Expr, Lit, parse_expr};

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

#[derive(Debug, Clone, PartialEq)]
pub struct TypeError {
    pub message: String,
}

impl TypeError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

pub fn check_expr(src: &str) -> Result<Type, TypeError> {
    let expr = parse_expr(src).map_err(|e| TypeError::new(format!("parse error: {}", e.message)))?;
    infer(&expr)
}

fn infer(expr: &Expr) -> Result<Type, TypeError> {
    match expr {
        Expr::Lit(Lit::Int(_)) => Ok(Type::Int),
        Expr::Lit(Lit::Float(_)) => Ok(Type::Float),
        Expr::Lit(Lit::Str(_)) => Ok(Type::String),
        Expr::Lit(Lit::Bool(_)) => Ok(Type::Bool),
        Expr::Lit(Lit::Unit) => Ok(Type::Unit),
        other => Err(TypeError::new(format!("cannot yet infer type of {other:?}"))),
    }
}
