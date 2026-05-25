//! Syntactic analysis for the Vela language.

use vela_lexer::{Keyword, Op, Punct, TokenKind, lex};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

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
    Match(Box<Expr>, Vec<MatchArm>),
    Record(Vec<(String, Expr)>),
    RecordUpdate(Box<Expr>, Vec<(String, Expr)>),
    Series(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pat: Pat,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pat {
    Wildcard,
    Var(String),
    Lit(Lit),
    Cons(String, Vec<Pat>),
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
    p.skip_newlines();
    let expr = p.parse_expr_bp(0)?;
    p.skip_newlines();
    if let Some(tok) = p.peek() {
        return Err(ParseError::new(format!("trailing token {tok:?}")));
    }
    Ok(expr)
}

pub fn parse_stmt(src: &str) -> Result<Stmt, ParseError> {
    let mut p = Parser::new(src);
    let stmt = p.parse_stmt()?;
    p.skip_newlines();
    if let Some(tok) = p.peek() {
        return Err(ParseError::new(format!("trailing token {tok:?}")));
    }
    Ok(stmt)
}

pub fn parse_program(src: &str) -> Result<Program, ParseError> {
    let mut p = Parser::new(src);
    let mut stmts = Vec::new();
    p.skip_newlines();
    while p.peek().is_some() {
        let stmt = p.parse_stmt()?;
        stmts.push(stmt);
        p.skip_newlines();
    }
    Ok(Program { stmts })
}

struct Parser {
    tokens: Vec<TokenKind>,
    pos: usize,
}

impl Parser {
    fn new(src: &str) -> Self {
        let tokens = lex(src)
            .map(|t| t.kind)
            .filter(|k| !matches!(k, TokenKind::Indent | TokenKind::Dedent))
            .collect();
        Self { tokens, pos: 0 }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Some(TokenKind::Newline)) {
            self.pos += 1;
        }
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

    fn parse_record(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Some(TokenKind::Punct(Punct::RBrace))) {
            self.bump();
            return Ok(Expr::Record(Vec::new()));
        }
        let first = self.parse_expr_bp(0)?;
        match self.peek() {
            Some(TokenKind::Keyword(Keyword::With)) => {
                self.bump();
                let fields = self.parse_field_list()?;
                self.expect(&TokenKind::Punct(Punct::RBrace))?;
                Ok(Expr::RecordUpdate(Box::new(first), fields))
            }
            Some(TokenKind::Op(Op::Assign)) => {
                let name = match first {
                    Expr::Var(n) => n,
                    other => {
                        return Err(ParseError::new(format!(
                            "expected field name before `=`, found {other:?}"
                        )));
                    }
                };
                self.bump();
                let value = self.parse_expr_bp(0)?;
                let mut fields = vec![(name, value)];
                while matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                    self.bump();
                    if matches!(self.peek(), Some(TokenKind::Punct(Punct::RBrace))) {
                        break;
                    }
                    let name = self.expect_ident()?;
                    self.expect(&TokenKind::Op(Op::Assign))?;
                    let value = self.parse_expr_bp(0)?;
                    fields.push((name, value));
                }
                self.expect(&TokenKind::Punct(Punct::RBrace))?;
                Ok(Expr::Record(fields))
            }
            other => Err(ParseError::new(format!(
                "expected `=` or `with` in record, found {other:?}"
            ))),
        }
    }

    fn parse_series(&mut self) -> Result<Expr, ParseError> {
        let mut elems = Vec::new();
        if matches!(self.peek(), Some(TokenKind::Punct(Punct::RBracket))) {
            self.bump();
            return Ok(Expr::Series(elems));
        }
        loop {
            elems.push(self.parse_expr_bp(0)?);
            if !matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                break;
            }
            self.bump();
            if matches!(self.peek(), Some(TokenKind::Punct(Punct::RBracket))) {
                break;
            }
        }
        self.expect(&TokenKind::Punct(Punct::RBracket))?;
        Ok(Expr::Series(elems))
    }

    fn parse_field_list(&mut self) -> Result<Vec<(String, Expr)>, ParseError> {
        let mut fields = Vec::new();
        loop {
            let name = self.expect_ident()?;
            self.expect(&TokenKind::Op(Op::Assign))?;
            let value = self.parse_expr_bp(0)?;
            fields.push((name, value));
            if !matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                break;
            }
            self.bump();
            if matches!(self.peek(), Some(TokenKind::Punct(Punct::RBrace))) {
                break;
            }
        }
        Ok(fields)
    }

    fn parse_pat(&mut self) -> Result<Pat, ParseError> {
        let tok = self.bump().ok_or_else(|| ParseError::new("expected pattern"))?;
        match tok {
            TokenKind::Int(n) => Ok(Pat::Lit(Lit::Int(n))),
            TokenKind::Float(f) => Ok(Pat::Lit(Lit::Float(f))),
            TokenKind::Str(s) => Ok(Pat::Lit(Lit::Str(s))),
            TokenKind::Bool(b) => Ok(Pat::Lit(Lit::Bool(b))),
            TokenKind::Ident(name) => {
                if name == "_" {
                    Ok(Pat::Wildcard)
                } else if name.starts_with(|c: char| c.is_ascii_uppercase()) {
                    let mut args = Vec::new();
                    while self.peek().is_some_and(starts_pat_atom) {
                        args.push(self.parse_pat_atom()?);
                    }
                    Ok(Pat::Cons(name, args))
                } else {
                    Ok(Pat::Var(name))
                }
            }
            other => Err(ParseError::new(format!("unexpected token in pattern: {other:?}"))),
        }
    }

    fn parse_pat_atom(&mut self) -> Result<Pat, ParseError> {
        let tok = self.bump().ok_or_else(|| ParseError::new("expected pattern atom"))?;
        match tok {
            TokenKind::Int(n) => Ok(Pat::Lit(Lit::Int(n))),
            TokenKind::Float(f) => Ok(Pat::Lit(Lit::Float(f))),
            TokenKind::Str(s) => Ok(Pat::Lit(Lit::Str(s))),
            TokenKind::Bool(b) => Ok(Pat::Lit(Lit::Bool(b))),
            TokenKind::Ident(name) => {
                if name == "_" {
                    Ok(Pat::Wildcard)
                } else if name.starts_with(|c: char| c.is_ascii_uppercase()) {
                    Ok(Pat::Cons(name, Vec::new()))
                } else {
                    Ok(Pat::Var(name))
                }
            }
            other => Err(ParseError::new(format!("unexpected token in pattern: {other:?}"))),
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
            TokenKind::Keyword(Keyword::Match) => {
                let scrut = self.parse_expr_bp(0)?;
                self.expect(&TokenKind::Keyword(Keyword::With))?;
                let mut arms = Vec::new();
                while matches!(self.peek(), Some(TokenKind::Punct(Punct::Bar))) {
                    self.bump();
                    let pat = self.parse_pat()?;
                    self.expect(&TokenKind::Op(Op::RArrow))?;
                    let body = self.parse_expr_bp(0)?;
                    arms.push(MatchArm { pat, body });
                }
                if arms.is_empty() {
                    return Err(ParseError::new("match expression has no arms"));
                }
                Ok(Expr::Match(Box::new(scrut), arms))
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
            TokenKind::Punct(Punct::LBrace) => self.parse_record(),
            TokenKind::Punct(Punct::LBracket) => self.parse_series(),
            other => Err(ParseError::new(format!("unexpected token: {other:?}"))),
        }
    }
}

const APP_BP: u8 = 25;

fn starts_pat_atom(tok: &TokenKind) -> bool {
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
    )
}

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
