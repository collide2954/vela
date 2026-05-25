//! Syntactic analysis for the Vela language.

use vela_lexer::{Punct, TokenKind, lex};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(Lit),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }
}

pub fn parse_expr(src: &str) -> Result<Expr, ParseError> {
    let mut tokens = lex(src)
        .map(|t| t.kind)
        .filter(|k| {
            !matches!(
                k,
                TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent
            )
        });
    let first = tokens.next().ok_or_else(|| ParseError::new("empty input"))?;
    let expr = match first {
        TokenKind::Int(n) => Expr::Lit(Lit::Int(n)),
        TokenKind::Float(f) => Expr::Lit(Lit::Float(f)),
        TokenKind::Str(s) => Expr::Lit(Lit::Str(s)),
        TokenKind::Bool(b) => Expr::Lit(Lit::Bool(b)),
        TokenKind::Punct(Punct::LParen) => match tokens.next() {
            Some(TokenKind::Punct(Punct::RParen)) => Expr::Lit(Lit::Unit),
            other => {
                return Err(ParseError::new(format!(
                    "expected `)` after `(`, found {other:?}"
                )));
            }
        },
        other => {
            return Err(ParseError::new(format!("unexpected token: {other:?}")));
        }
    };
    Ok(expr)
}
