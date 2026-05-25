//! Syntactic analysis for the Vela language.

use vela_lexer::{Punct, TokenKind, lex};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(Lit),
    Var(String),
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
    let mut p = Parser::new(src);
    let expr = p.parse_expr()?;
    if let Some(tok) = p.peek() {
        return Err(ParseError::new(format!("trailing token {tok:?}")));
    }
    Ok(expr)
}

struct Parser {
    tokens: Vec<TokenKind>,
    pos: usize,
}

impl Parser {
    fn new(src: &str) -> Self {
        let tokens = lex(src)
            .map(|t| t.kind)
            .filter(|k| !matches!(k, TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent))
            .collect();
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos)
    }

    fn bump(&mut self) -> Option<TokenKind> {
        let tok = self.tokens.get(self.pos).cloned()?;
        self.pos += 1;
        Some(tok)
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<(), ParseError> {
        match self.bump() {
            Some(ref t) if t == expected => Ok(()),
            Some(other) => {
                Err(ParseError::new(format!("expected {expected:?}, found {other:?}")))
            }
            None => Err(ParseError::new(format!("expected {expected:?}, found end of input"))),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_atom()
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let tok = self.bump().ok_or_else(|| ParseError::new("empty input"))?;
        match tok {
            TokenKind::Int(n) => Ok(Expr::Lit(Lit::Int(n))),
            TokenKind::Float(f) => Ok(Expr::Lit(Lit::Float(f))),
            TokenKind::Str(s) => Ok(Expr::Lit(Lit::Str(s))),
            TokenKind::Bool(b) => Ok(Expr::Lit(Lit::Bool(b))),
            TokenKind::Ident(name) => Ok(Expr::Var(name)),
            TokenKind::Punct(Punct::LParen) => {
                if matches!(self.peek(), Some(TokenKind::Punct(Punct::RParen))) {
                    self.bump();
                    return Ok(Expr::Lit(Lit::Unit));
                }
                let inner = self.parse_expr()?;
                self.expect(&TokenKind::Punct(Punct::RParen))?;
                Ok(inner)
            }
            other => Err(ParseError::new(format!("unexpected token: {other:?}"))),
        }
    }
}
