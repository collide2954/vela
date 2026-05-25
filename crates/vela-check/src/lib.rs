//! Type checking and inference for the Vela language.

use vela_parser::{BinOp, Expr, Lit, UnOp, parse_expr};

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
        Expr::UnaryOp(op, inner) => infer_unary(*op, inner),
        Expr::BinOp(op, lhs, rhs) => infer_binary(*op, lhs, rhs),
        other => Err(TypeError::new(format!("cannot yet infer type of {other:?}"))),
    }
}

fn infer_unary(op: UnOp, inner: &Expr) -> Result<Type, TypeError> {
    let t = infer(inner)?;
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

fn infer_binary(op: BinOp, lhs: &Expr, rhs: &Expr) -> Result<Type, TypeError> {
    let l = infer(lhs)?;
    let r = infer(rhs)?;
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
