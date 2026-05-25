//! Syntactic analysis for the Vela language.

use vela_lexer::{Keyword, Op, Punct, TokenKind, lex};

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let { name: String, params: Vec<String>, body: Expr },
    Var { name: String, body: Expr },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(Lit),
    Var(String),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    UnaryOp(UnOp, Box<Expr>),
    Postfix(PostOp, Box<Expr>),
    App(Box<Expr>, Box<Expr>),
    Lambda(Vec<String>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Lit {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Unit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Pipe,
    Tilde,
    Or,
    And,
    Eq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Concat,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostOp {
    Question,
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
    let expr = p.parse_expr_bp(0)?;
    if let Some(tok) = p.peek() {
        return Err(ParseError::new(format!("trailing token {tok:?}")));
    }
    Ok(expr)
}

pub fn parse_stmt(src: &str) -> Result<Stmt, ParseError> {
    let mut p = Parser::new(src);
    let stmt = p.parse_stmt()?;
    if let Some(tok) = p.peek() {
        return Err(ParseError::new(format!("trailing token {tok:?}")));
    }
    Ok(stmt)
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

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Some(TokenKind::Keyword(Keyword::Let)) => {
                self.bump();
                let name = self.expect_ident()?;
                let mut params = Vec::new();
                while let Some(TokenKind::Ident(_)) = self.peek() {
                    params.push(self.expect_ident()?);
                }
                self.expect(&TokenKind::Op(Op::Assign))?;
                let body = self.parse_expr_bp(0)?;
                Ok(Stmt::Let { name, params, body })
            }
            Some(TokenKind::Keyword(Keyword::Var)) => {
                self.bump();
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Op(Op::Assign))?;
                let body = self.parse_expr_bp(0)?;
                Ok(Stmt::Var { name, body })
            }
            _ => Ok(Stmt::Expr(self.parse_expr_bp(0)?)),
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.bump() {
            Some(TokenKind::Ident(name)) => Ok(name),
            Some(other) => Err(ParseError::new(format!("expected identifier, found {other:?}"))),
            None => Err(ParseError::new("expected identifier, found end of input")),
        }
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = if let Some((op, r_bp)) = self.peek().and_then(prefix_op) {
            self.bump();
            let rhs = self.parse_expr_bp(r_bp)?;
            Expr::UnaryOp(op, Box::new(rhs))
        } else {
            self.parse_atom()?
        };

        loop {
            if let Some((op, l_bp)) = self.peek().and_then(postfix_op) {
                if l_bp < min_bp {
                    break;
                }
                self.bump();
                lhs = Expr::Postfix(op, Box::new(lhs));
                continue;
            }
            if self.peek().is_some_and(starts_atom) && APP_BP >= min_bp {
                let rhs = self.parse_expr_bp(APP_BP + 1)?;
                lhs = Expr::App(Box::new(lhs), Box::new(rhs));
                continue;
            }
            let Some((op, l_bp, r_bp)) = self.peek().and_then(binary_op) else { break };
            if l_bp < min_bp {
                break;
            }
            self.bump();
            let rhs = self.parse_expr_bp(r_bp)?;
            lhs = Expr::BinOp(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let tok = self.bump().ok_or_else(|| ParseError::new("empty input"))?;
        match tok {
            TokenKind::Int(n) => Ok(Expr::Lit(Lit::Int(n))),
            TokenKind::Float(f) => Ok(Expr::Lit(Lit::Float(f))),
            TokenKind::Str(s) => Ok(Expr::Lit(Lit::Str(s))),
            TokenKind::Bool(b) => Ok(Expr::Lit(Lit::Bool(b))),
            TokenKind::Ident(name) => Ok(Expr::Var(name)),
            TokenKind::Keyword(Keyword::Fn) => {
                let mut params = Vec::new();
                while let Some(TokenKind::Ident(_)) = self.peek() {
                    params.push(self.expect_ident()?);
                }
                self.expect(&TokenKind::Op(Op::RArrow))?;
                let body = self.parse_expr_bp(0)?;
                Ok(Expr::Lambda(params, Box::new(body)))
            }
            TokenKind::Keyword(Keyword::If) => {
                let cond = self.parse_expr_bp(0)?;
                self.expect(&TokenKind::Keyword(Keyword::Then))?;
                let then_b = self.parse_expr_bp(0)?;
                self.expect(&TokenKind::Keyword(Keyword::Else))?;
                let else_b = self.parse_expr_bp(0)?;
                Ok(Expr::If(Box::new(cond), Box::new(then_b), Box::new(else_b)))
            }
            TokenKind::Punct(Punct::LParen) => {
                if matches!(self.peek(), Some(TokenKind::Punct(Punct::RParen))) {
                    self.bump();
                    return Ok(Expr::Lit(Lit::Unit));
                }
                let inner = self.parse_expr_bp(0)?;
                self.expect(&TokenKind::Punct(Punct::RParen))?;
                Ok(inner)
            }
            other => Err(ParseError::new(format!("unexpected token: {other:?}"))),
        }
    }
}

const APP_BP: u8 = 25;

fn starts_atom(tok: &TokenKind) -> bool {
    matches!(
        tok,
        TokenKind::Int(_)
            | TokenKind::UInt(_)
            | TokenKind::BigInt(_)
            | TokenKind::Float(_)
            | TokenKind::Decimal(_)
            | TokenKind::Str(_)
            | TokenKind::Bool(_)
            | TokenKind::Ident(_)
            | TokenKind::Sym(_)
            | TokenKind::Punct(
                Punct::LParen | Punct::LBracket | Punct::LBrace | Punct::ArrayOpen | Punct::FrameOpen,
            )
    )
}

fn prefix_op(tok: &TokenKind) -> Option<(UnOp, u8)> {
    let op = match tok {
        TokenKind::Op(Op::Minus) => UnOp::Neg,
        TokenKind::Keyword(Keyword::Not) => UnOp::Not,
        _ => return None,
    };
    Some((op, 19))
}

fn postfix_op(tok: &TokenKind) -> Option<(PostOp, u8)> {
    match tok {
        TokenKind::Op(Op::Question) => Some((PostOp::Question, 21)),
        _ => None,
    }
}

fn binary_op(tok: &TokenKind) -> Option<(BinOp, u8, u8)> {
    Some(match tok {
        TokenKind::Op(Op::Pipe) => (BinOp::Pipe, 1, 2),
        TokenKind::Op(Op::Tilde) => (BinOp::Tilde, 3, 4),
        TokenKind::Keyword(Keyword::Or) => (BinOp::Or, 5, 6),
        TokenKind::Keyword(Keyword::And) => (BinOp::And, 7, 8),
        TokenKind::Op(Op::Eq) => (BinOp::Eq, 9, 10),
        TokenKind::Op(Op::NotEq) => (BinOp::NotEq, 9, 10),
        TokenKind::Op(Op::Lt) => (BinOp::Lt, 9, 10),
        TokenKind::Op(Op::Le) => (BinOp::Le, 9, 10),
        TokenKind::Op(Op::Gt) => (BinOp::Gt, 9, 10),
        TokenKind::Op(Op::Ge) => (BinOp::Ge, 9, 10),
        TokenKind::Op(Op::PlusPlus) => (BinOp::Concat, 11, 12),
        TokenKind::Op(Op::Plus) => (BinOp::Add, 13, 14),
        TokenKind::Op(Op::Minus) => (BinOp::Sub, 13, 14),
        TokenKind::Op(Op::Star) => (BinOp::Mul, 15, 16),
        TokenKind::Op(Op::Slash) => (BinOp::Div, 15, 16),
        TokenKind::Op(Op::Percent) => (BinOp::Mod, 15, 16),
        TokenKind::Op(Op::Caret) => (BinOp::Pow, 18, 17),
        _ => return None,
    })
}
