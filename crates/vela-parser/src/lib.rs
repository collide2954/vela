//! Syntactic analysis for the Vela language.

use std::ops::Range;
use vela_lexer::{Keyword, Op, Punct, Span, Token, TokenKind, lex};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let { name: String, params: Vec<Param>, return_ty: Option<Ty>, body: Expr },
    Var { name: String, ty: Option<Ty>, body: Expr },
    Mutate { name: String, body: Expr },
    For { binding: String, iter: Expr, body: Expr },
    Destructure { pat: Pat, body: Expr },
    TypeDecl(TypeDecl),
    TraitDecl(TraitDecl),
    Impl(ImplBlock),
    Tests(Vec<TestCase>),
    Extern { abi: String, signatures: Vec<TraitMethodSig> },
    Input { name: String, body: Expr },
    Output { name: String, body: Expr },
    Import { path: Vec<String>, kind: ImportKind, public: bool },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestCase {
    Test { name: String, body: Expr },
    Prop { name: String, params: Vec<Param>, guard: Option<Expr>, body: Expr },
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitDecl {
    pub name: String,
    pub type_var: String,
    pub methods: Vec<TraitMethodSig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethodSig {
    pub name: String,
    pub params: Vec<Param>,
    pub return_ty: Ty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    pub trait_name: String,
    pub ty: Ty,
    pub methods: Vec<ImplMethod>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImplMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_ty: Option<Ty>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub pat: Pat,
    pub ty: Option<Ty>,
}

impl Param {
    pub fn simple_name(&self) -> Option<&str> {
        if let Pat::Var(n) = &self.pat { Some(n) } else { None }
    }
}

impl From<&str> for Param {
    fn from(name: &str) -> Self {
        Param { pat: Pat::Var(name.into()), ty: None }
    }
}

impl From<String> for Param {
    fn from(name: String) -> Self {
        Param { pat: Pat::Var(name), ty: None }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportKind {
    All,
    Items(Vec<String>),
    Alias(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: TypeDeclBody,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeDeclBody {
    Sum(Vec<TypeVariant>),
    Alias(Ty),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeVariant {
    pub name: String,
    pub args: Vec<Ty>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ty {
    Unit,
    Con(String),
    Var(String),
    App(Box<Ty>, Vec<Ty>),
    Record(Vec<(String, Ty)>),
    Series(Box<Ty>),
    Tuple(Vec<Ty>),
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
    Field(Box<Expr>, String),
    Tuple(Vec<Expr>),
    DataFrameLit(Vec<(String, Expr)>),
    ArrayLit(Vec<Vec<Expr>>),
    Sym(String),
    Block { stmts: Vec<Stmt>, trailing: Option<Box<Expr>> },
    Scope(Box<Expr>),
    Spawn(Box<Expr>),
    AppBlock(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pat: Pat,
    pub guard: Option<Expr>,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pat {
    Wildcard,
    Var(String),
    Lit(Lit),
    Cons(String, Vec<Pat>),
    Or(Vec<Pat>),
    As(Box<Pat>, String),
    List(Vec<ListPart>),
    Range(Box<Pat>, Box<Pat>),
    Tuple(Vec<Pat>),
    Record(Vec<(String, Pat)>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ListPart {
    Pat(Pat),
    Rest(Option<String>),
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
    pub span: Option<Range<usize>>,
    pub code: &'static str,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), span: None, code: "E0001" }
    }

    fn with_span(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
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
    tokens: Vec<Token>,
    pos: usize,
    eof_offset: usize,
}

impl Parser {
    fn new(src: &str) -> Self {
        let tokens: Vec<Token> = lex(src).collect();
        Self { tokens, pos: 0, eof_offset: src.len() }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Some(TokenKind::Newline)) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<&TokenKind> {
        self.tokens.get(self.pos).map(|t| &t.kind)
    }

    fn current_span(&self) -> Span {
        match self.tokens.get(self.pos) {
            Some(t) => t.span.clone(),
            None => self.eof_offset..self.eof_offset,
        }
    }

    fn bump(&mut self) -> Option<TokenKind> {
        let tok = self.tokens.get(self.pos).map(|t| t.kind.clone())?;
        self.pos += 1;
        Some(tok)
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<(), ParseError> {
        let span = self.current_span();
        match self.bump() {
            Some(ref t) if t == expected => Ok(()),
            Some(other) => Err(ParseError::new(format!(
                "expected {expected}, found {other}"
            ))
            .with_span(span)),
            None => Err(ParseError::new(format!(
                "expected {expected}, found end of input"
            ))
            .with_span(span)),
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek() {
            Some(TokenKind::Keyword(Keyword::Let)) => {
                self.bump();
                if matches!(
                    self.peek(),
                    Some(TokenKind::Punct(
                        Punct::LParen | Punct::LBrace | Punct::LBracket,
                    ))
                ) {
                    let pat = self.parse_pat()?;
                    self.expect(&TokenKind::Op(Op::Assign))?;
                    let body = self.parse_body_after_block_intro()?;
                    return Ok(Stmt::Destructure { pat, body });
                }
                let name = self.expect_ident()?;
                let mut params = Vec::new();
                loop {
                    match self.peek() {
                        Some(TokenKind::Ident(_)) => {
                            let n = self.expect_ident()?;
                            params.push(Param { pat: Pat::Var(n), ty: None });
                        }
                        Some(TokenKind::Punct(Punct::LParen)) => {
                            let save = self.pos;
                            self.bump();
                            if let Some(p) = self.try_typed_param() {
                                params.push(p);
                            } else {
                                self.pos = save;
                                if let Some(p) = self.try_pattern_param() {
                                    params.push(p);
                                } else {
                                    break;
                                }
                            }
                        }
                        Some(TokenKind::Punct(Punct::LBrace)) => {
                            if let Some(p) = self.try_pattern_param() {
                                params.push(p);
                            } else {
                                break;
                            }
                        }
                        _ => break,
                    }
                }
                let return_ty = if matches!(self.peek(), Some(TokenKind::Punct(Punct::Colon))) {
                    self.bump();
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(&TokenKind::Op(Op::Assign))?;
                let body = self.parse_body_after_block_intro()?;
                Ok(Stmt::Let { name, params, return_ty, body })
            }
            Some(TokenKind::Keyword(Keyword::Var)) => {
                self.bump();
                let name = self.expect_ident()?;
                let ty = if matches!(self.peek(), Some(TokenKind::Punct(Punct::Colon))) {
                    self.bump();
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(&TokenKind::Op(Op::Assign))?;
                let body = self.parse_body_after_block_intro()?;
                Ok(Stmt::Var { name, ty, body })
            }
            Some(TokenKind::Keyword(Keyword::For)) => {
                self.bump();
                let binding = self.expect_ident()?;
                self.expect(&TokenKind::Keyword(Keyword::In))?;
                let iter = self.parse_expr_bp(0)?;
                self.expect(&TokenKind::Punct(Punct::Colon))?;
                let body = self.parse_body_after_block_intro()?;
                Ok(Stmt::For { binding, iter, body })
            }
            Some(TokenKind::Keyword(Keyword::Pub)) => {
                self.bump();
                if matches!(self.peek(), Some(TokenKind::Keyword(Keyword::Import))) {
                    self.parse_import(true)
                } else {
                    self.parse_stmt()
                }
            }
            Some(TokenKind::Keyword(Keyword::Import)) => self.parse_import(false),
            Some(TokenKind::Keyword(Keyword::Tests)) => {
                self.bump();
                self.expect(&TokenKind::Op(Op::Assign))?;
                let cases = self.parse_test_cases()?;
                Ok(Stmt::Tests(cases))
            }
            Some(TokenKind::Keyword(Keyword::Extern)) => {
                self.bump();
                let abi = self.expect_string()?;
                self.expect(&TokenKind::Op(Op::Assign))?;
                let signatures = self.parse_trait_methods()?;
                Ok(Stmt::Extern { abi, signatures })
            }
            Some(TokenKind::Keyword(Keyword::Input)) => {
                self.bump();
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Op(Op::Assign))?;
                let body = self.parse_expr_bp(0)?;
                Ok(Stmt::Input { name, body })
            }
            Some(TokenKind::Keyword(Keyword::Output)) => {
                self.bump();
                let name = self.expect_ident()?;
                self.expect(&TokenKind::Op(Op::Assign))?;
                let body = self.parse_expr_bp(0)?;
                Ok(Stmt::Output { name, body })
            }
            Some(TokenKind::Keyword(Keyword::Trait)) => {
                self.bump();
                let name = self.expect_ident()?;
                let type_var = self.expect_ident()?;
                self.expect(&TokenKind::Op(Op::Assign))?;
                let methods = self.parse_trait_methods()?;
                Ok(Stmt::TraitDecl(TraitDecl { name, type_var, methods }))
            }
            Some(TokenKind::Keyword(Keyword::Impl)) => {
                self.bump();
                let trait_name = self.expect_ident()?;
                let ty = self.parse_type_atom()?;
                self.expect(&TokenKind::Op(Op::Assign))?;
                let methods = self.parse_impl_methods()?;
                Ok(Stmt::Impl(ImplBlock { trait_name, ty, methods }))
            }
            Some(TokenKind::Keyword(Keyword::Type)) => {
                self.bump();
                let name = self.expect_ident()?;
                let mut params = Vec::new();
                while matches!(self.peek(), Some(TokenKind::Punct(Punct::Tick))) {
                    self.bump();
                    params.push(self.expect_ident()?);
                }
                self.expect(&TokenKind::Op(Op::Assign))?;
                let body = self.parse_type_decl_body()?;
                Ok(Stmt::TypeDecl(TypeDecl { name, params, body }))
            }
            _ => {
                let expr = self.parse_expr_bp(0)?;
                if matches!(self.peek(), Some(TokenKind::Op(Op::LArrow))) {
                    self.bump();
                    let value = self.parse_expr_bp(0)?;
                    match expr {
                        Expr::Var(name) => Ok(Stmt::Mutate { name, body: value }),
                        other => Err(ParseError::new(format!(
                            "expected variable name on left of `<-`, found {other:?}"
                        ))),
                    }
                } else {
                    Ok(Stmt::Expr(expr))
                }
            }
        }
    }

    fn parse_test_cases(&mut self) -> Result<Vec<TestCase>, ParseError> {
        self.skip_newlines();
        let dedent_pending = matches!(self.peek(), Some(TokenKind::Indent));
        if dedent_pending {
            self.bump();
            self.skip_newlines();
        }
        let mut cases = Vec::new();
        loop {
            match self.peek() {
                Some(TokenKind::Keyword(Keyword::Test)) => {
                    self.bump();
                    let name = self.expect_string()?;
                    self.expect(&TokenKind::Op(Op::Assign))?;
                    let body = self.parse_body_after_block_intro()?;
                    cases.push(TestCase::Test { name, body });
                }
                Some(TokenKind::Keyword(Keyword::Prop)) => {
                    self.bump();
                    let name = self.expect_string()?;
                    let mut params = Vec::new();
                    loop {
                        match self.peek() {
                            Some(TokenKind::Ident(_)) => {
                                let n = self.expect_ident()?;
                                params.push(Param { pat: Pat::Var(n), ty: None });
                            }
                            Some(TokenKind::Punct(Punct::LParen)) => {
                                let save = self.pos;
                                self.bump();
                                if let Some(p) = self.try_typed_param() {
                                    params.push(p);
                                } else {
                                    self.pos = save;
                                    break;
                                }
                            }
                            _ => break,
                        }
                    }
                    let guard = if matches!(self.peek(), Some(TokenKind::Keyword(Keyword::When))) {
                        self.bump();
                        Some(self.parse_expr_bp(0)?)
                    } else {
                        None
                    };
                    self.expect(&TokenKind::Op(Op::Assign))?;
                    let body = self.parse_body_after_block_intro()?;
                    cases.push(TestCase::Prop { name, params, guard, body });
                }
                _ => break,
            }
            self.skip_newlines();
        }
        if dedent_pending && matches!(self.peek(), Some(TokenKind::Dedent)) {
            self.bump();
        }
        Ok(cases)
    }

    fn expect_string(&mut self) -> Result<String, ParseError> {
        match self.bump() {
            Some(TokenKind::Str(s)) => Ok(s),
            Some(other) => {
                Err(ParseError::new(format!("expected string literal, found {other}")))
            }
            None => Err(ParseError::new("expected string literal, found end of input")),
        }
    }

    fn parse_trait_methods(&mut self) -> Result<Vec<TraitMethodSig>, ParseError> {
        self.skip_newlines();
        let dedent_pending = matches!(self.peek(), Some(TokenKind::Indent));
        if dedent_pending {
            self.bump();
            self.skip_newlines();
        }
        let mut methods = Vec::new();
        while matches!(self.peek(), Some(TokenKind::Keyword(Keyword::Fn))) {
            self.bump();
            let name = self.expect_ident()?;
            let params = self.parse_typed_param_list()?;
            self.expect(&TokenKind::Punct(Punct::Colon))?;
            let return_ty = self.parse_type()?;
            methods.push(TraitMethodSig { name, params, return_ty });
            self.skip_newlines();
        }
        if dedent_pending && matches!(self.peek(), Some(TokenKind::Dedent)) {
            self.bump();
        }
        Ok(methods)
    }

    fn parse_impl_methods(&mut self) -> Result<Vec<ImplMethod>, ParseError> {
        self.skip_newlines();
        let dedent_pending = matches!(self.peek(), Some(TokenKind::Indent));
        if dedent_pending {
            self.bump();
            self.skip_newlines();
        }
        let mut methods = Vec::new();
        while matches!(self.peek(), Some(TokenKind::Keyword(Keyword::Fn))) {
            self.bump();
            let name = self.expect_ident()?;
            let mut params = Vec::new();
            loop {
                match self.peek() {
                    Some(TokenKind::Ident(_)) => {
                        let n = self.expect_ident()?;
                        params.push(Param { pat: Pat::Var(n), ty: None });
                    }
                    Some(TokenKind::Punct(Punct::LParen)) => {
                        let save = self.pos;
                        self.bump();
                        if let Some(p) = self.try_typed_param() {
                            params.push(p);
                        } else {
                            self.pos = save;
                            break;
                        }
                    }
                    _ => break,
                }
            }
            let return_ty = if matches!(self.peek(), Some(TokenKind::Punct(Punct::Colon))) {
                self.bump();
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect(&TokenKind::Op(Op::Assign))?;
            let body = self.parse_body_after_block_intro()?;
            methods.push(ImplMethod { name, params, return_ty, body });
            self.skip_newlines();
        }
        if dedent_pending && matches!(self.peek(), Some(TokenKind::Dedent)) {
            self.bump();
        }
        Ok(methods)
    }

    fn parse_typed_param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        while matches!(self.peek(), Some(TokenKind::Punct(Punct::LParen))) {
            let save = self.pos;
            self.bump();
            if let Some(p) = self.try_typed_param() {
                params.push(p);
            } else {
                self.pos = save;
                break;
            }
        }
        Ok(params)
    }

    fn try_pattern_param(&mut self) -> Option<Param> {
        let save = self.pos;
        let pat = self.parse_pat_atom().ok()?;
        if !is_irrefutable_param_pat(&pat) {
            self.pos = save;
            return None;
        }
        Some(Param { pat, ty: None })
    }

    fn try_typed_param(&mut self) -> Option<Param> {
        let save = self.pos;
        let name = match self.peek() {
            Some(TokenKind::Ident(_)) => self.expect_ident().ok()?,
            _ => return None,
        };
        if !matches!(self.peek(), Some(TokenKind::Punct(Punct::Colon))) {
            self.pos = save;
            return None;
        }
        self.bump();
        let ty = self.parse_type().ok()?;
        if !matches!(self.peek(), Some(TokenKind::Punct(Punct::RParen))) {
            self.pos = save;
            return None;
        }
        self.bump();
        Some(Param { pat: Pat::Var(name), ty: Some(ty) })
    }

    fn parse_import(&mut self, public: bool) -> Result<Stmt, ParseError> {
        self.bump();
        let first = self.expect_ident()?;
        let mut path = vec![first];
        while matches!(self.peek(), Some(TokenKind::Op(Op::Dot))) {
            self.bump();
            path.push(self.expect_ident()?);
        }
        let kind = match self.peek() {
            Some(TokenKind::Punct(Punct::LParen)) => {
                self.bump();
                let mut items = Vec::new();
                if !matches!(self.peek(), Some(TokenKind::Punct(Punct::RParen))) {
                    loop {
                        items.push(self.expect_ident()?);
                        if !matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                            break;
                        }
                        self.bump();
                        if matches!(self.peek(), Some(TokenKind::Punct(Punct::RParen))) {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::Punct(Punct::RParen))?;
                ImportKind::Items(items)
            }
            Some(TokenKind::Keyword(Keyword::As)) => {
                self.bump();
                ImportKind::Alias(self.expect_ident()?)
            }
            _ => ImportKind::All,
        };
        Ok(Stmt::Import { path, kind, public })
    }

    fn parse_type_decl_body(&mut self) -> Result<TypeDeclBody, ParseError> {
        self.skip_newlines();
        let dedent_pending = matches!(self.peek(), Some(TokenKind::Indent));
        if dedent_pending {
            self.bump();
            self.skip_newlines();
        }
        let body = match self.peek() {
            Some(TokenKind::Punct(Punct::Bar)) => {
                TypeDeclBody::Sum(self.parse_sum_variants()?)
            }
            Some(TokenKind::Ident(name)) if starts_with_uppercase(name) => {
                let variant = self.parse_variant_no_bar()?;
                TypeDeclBody::Sum(vec![variant])
            }
            _ => TypeDeclBody::Alias(self.parse_type()?),
        };
        self.skip_newlines();
        if dedent_pending && matches!(self.peek(), Some(TokenKind::Dedent)) {
            self.bump();
        }
        Ok(body)
    }

    fn parse_sum_variants(&mut self) -> Result<Vec<TypeVariant>, ParseError> {
        let mut variants = Vec::new();
        while matches!(self.peek(), Some(TokenKind::Punct(Punct::Bar))) {
            self.bump();
            variants.push(self.parse_variant_no_bar()?);
            self.skip_newlines();
        }
        Ok(variants)
    }

    fn parse_variant_no_bar(&mut self) -> Result<TypeVariant, ParseError> {
        let name = self.expect_ident()?;
        if !starts_with_uppercase(&name) {
            return Err(ParseError::new(format!(
                "variant name must begin with uppercase, found `{name}`"
            )));
        }
        let mut args = Vec::new();
        while self.peek().is_some_and(starts_type_atom) {
            args.push(self.parse_type_atom()?);
        }
        Ok(TypeVariant { name, args })
    }

    fn parse_type(&mut self) -> Result<Ty, ParseError> {
        let mut t = self.parse_type_atom()?;
        while self.peek().is_some_and(starts_type_atom) {
            let arg = self.parse_type_atom()?;
            t = match t {
                Ty::App(base, mut args) => {
                    args.push(arg);
                    Ty::App(base, args)
                }
                _ => Ty::App(Box::new(t), vec![arg]),
            };
        }
        Ok(t)
    }

    fn parse_type_atom(&mut self) -> Result<Ty, ParseError> {
        match self.peek() {
            Some(TokenKind::Ident(_)) => {
                let name = self.expect_ident()?;
                let con = Ty::Con(name);
                self.maybe_bracket_type_app(con)
            }
            Some(TokenKind::Punct(Punct::Tick)) => {
                self.bump();
                let name = self.expect_ident()?;
                let var = Ty::Var(name);
                self.maybe_bracket_type_app(var)
            }
            Some(TokenKind::Punct(Punct::LParen)) => {
                self.bump();
                if matches!(self.peek(), Some(TokenKind::Punct(Punct::RParen))) {
                    self.bump();
                    return Ok(Ty::Unit);
                }
                let first = self.parse_type()?;
                if matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                    let mut elems = vec![first];
                    while matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                        self.bump();
                        if matches!(self.peek(), Some(TokenKind::Punct(Punct::RParen))) {
                            break;
                        }
                        elems.push(self.parse_type()?);
                    }
                    self.expect(&TokenKind::Punct(Punct::RParen))?;
                    Ok(Ty::Tuple(elems))
                } else {
                    self.expect(&TokenKind::Punct(Punct::RParen))?;
                    Ok(first)
                }
            }
            Some(TokenKind::Punct(Punct::LBrace)) => {
                self.bump();
                let fields = self.parse_type_record_fields()?;
                self.expect(&TokenKind::Punct(Punct::RBrace))?;
                Ok(Ty::Record(fields))
            }
            Some(TokenKind::Punct(Punct::LBracket)) => {
                self.bump();
                let inner = self.parse_type()?;
                self.expect(&TokenKind::Punct(Punct::RBracket))?;
                Ok(Ty::Series(Box::new(inner)))
            }
            other => Err(ParseError::new(format!("expected type, found {other:?}"))),
        }
    }

    fn maybe_bracket_type_app(&mut self, base: Ty) -> Result<Ty, ParseError> {
        if !matches!(self.peek(), Some(TokenKind::Punct(Punct::LBracket))) {
            return Ok(base);
        }
        self.bump();
        let mut args = Vec::new();
        if matches!(self.peek(), Some(TokenKind::Punct(Punct::RBracket))) {
            self.bump();
            return Ok(Ty::App(Box::new(base), args));
        }
        loop {
            if matches!(self.peek(), Some(TokenKind::Int(_))) {
                self.bump();
                args.push(Ty::Con("_dim".into()));
            } else {
                args.push(self.parse_type()?);
            }
            if !matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                break;
            }
            self.bump();
            if matches!(self.peek(), Some(TokenKind::Punct(Punct::RBracket))) {
                break;
            }
        }
        self.expect(&TokenKind::Punct(Punct::RBracket))?;
        Ok(Ty::App(Box::new(base), args))
    }

    fn parse_type_record_fields(&mut self) -> Result<Vec<(String, Ty)>, ParseError> {
        let mut fields = Vec::new();
        if matches!(self.peek(), Some(TokenKind::Punct(Punct::RBrace))) {
            return Ok(fields);
        }
        loop {
            let name = self.expect_ident()?;
            self.expect(&TokenKind::Punct(Punct::Colon))?;
            let ty = self.parse_type()?;
            fields.push((name, ty));
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

    fn parse_body_after_block_intro(&mut self) -> Result<Expr, ParseError> {
        if matches!(self.peek(), Some(TokenKind::Newline)) {
            let save = self.pos;
            self.bump();
            if matches!(self.peek(), Some(TokenKind::Indent)) {
                self.bump();
                return self.parse_block_contents();
            }
            self.pos = save;
        }
        self.parse_expr_bp(0)
    }

    fn parse_block_contents(&mut self) -> Result<Expr, ParseError> {
        let mut stmts = Vec::new();
        loop {
            self.skip_newlines();
            if matches!(self.peek(), Some(TokenKind::Dedent) | None) {
                break;
            }
            stmts.push(self.parse_stmt()?);
        }
        if matches!(self.peek(), Some(TokenKind::Dedent)) {
            self.bump();
        }
        Ok(flatten_block(stmts))
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

    fn parse_array(&mut self) -> Result<Expr, ParseError> {
        let mut rows: Vec<Vec<Expr>> = Vec::new();
        if matches!(self.peek(), Some(TokenKind::Punct(Punct::ArrayClose))) {
            self.bump();
            return Ok(Expr::ArrayLit(rows));
        }
        let mut row: Vec<Expr> = Vec::new();
        loop {
            row.push(self.parse_expr_bp(0)?);
            match self.peek() {
                Some(TokenKind::Punct(Punct::Comma)) => {
                    self.bump();
                }
                Some(TokenKind::Punct(Punct::Semi)) => {
                    self.bump();
                    rows.push(std::mem::take(&mut row));
                }
                _ => break,
            }
        }
        if !row.is_empty() {
            rows.push(row);
        }
        self.expect(&TokenKind::Punct(Punct::ArrayClose))?;
        Ok(Expr::ArrayLit(rows))
    }

    fn parse_dataframe(&mut self) -> Result<Expr, ParseError> {
        let mut cols = Vec::new();
        if matches!(self.peek(), Some(TokenKind::Punct(Punct::FrameClose))) {
            self.bump();
            return Ok(Expr::DataFrameLit(cols));
        }
        loop {
            let name = self.expect_ident()?;
            self.expect(&TokenKind::Punct(Punct::Colon))?;
            let value = self.parse_expr_bp(0)?;
            cols.push((name, value));
            if !matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                break;
            }
            self.bump();
            if matches!(self.peek(), Some(TokenKind::Punct(Punct::FrameClose))) {
                break;
            }
        }
        self.expect(&TokenKind::Punct(Punct::FrameClose))?;
        Ok(Expr::DataFrameLit(cols))
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

    fn parse_arm_pat(&mut self) -> Result<Pat, ParseError> {
        let mut alts = vec![self.parse_as_pat()?];
        while matches!(self.peek(), Some(TokenKind::Punct(Punct::Bar))) {
            let save = self.pos;
            self.bump();
            match self.parse_as_pat() {
                Ok(p) => alts.push(p),
                Err(_) => {
                    self.pos = save;
                    break;
                }
            }
        }
        Ok(if alts.len() == 1 {
            alts.pop().expect("nonempty")
        } else {
            Pat::Or(alts)
        })
    }

    fn parse_as_pat(&mut self) -> Result<Pat, ParseError> {
        let p = self.parse_pat()?;
        if matches!(self.peek(), Some(TokenKind::Keyword(Keyword::As))) {
            self.bump();
            let name = self.expect_ident()?;
            Ok(Pat::As(Box::new(p), name))
        } else {
            Ok(p)
        }
    }

    fn parse_pat(&mut self) -> Result<Pat, ParseError> {
        if let Some(TokenKind::Ident(name)) = self.peek()
            && name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
        {
            let name = self.expect_ident()?;
            let mut args = Vec::new();
            while self.peek().is_some_and(starts_pat_atom) {
                args.push(self.parse_pat_atom()?);
            }
            return Ok(Pat::Cons(name, args));
        }
        let lo = self.parse_pat_atom()?;
        if matches!(self.peek(), Some(TokenKind::Op(Op::DotDotEq))) {
            self.bump();
            let hi = self.parse_pat_atom()?;
            return Ok(Pat::Range(Box::new(lo), Box::new(hi)));
        }
        Ok(lo)
    }

    fn parse_paren_pat_after_lparen(&mut self) -> Result<Pat, ParseError> {
        if matches!(self.peek(), Some(TokenKind::Punct(Punct::RParen))) {
            self.bump();
            return Ok(Pat::Lit(Lit::Unit));
        }
        let first = self.parse_pat()?;
        if matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
            let mut elems = vec![first];
            while matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                self.bump();
                if matches!(self.peek(), Some(TokenKind::Punct(Punct::RParen))) {
                    break;
                }
                elems.push(self.parse_pat()?);
            }
            self.expect(&TokenKind::Punct(Punct::RParen))?;
            Ok(Pat::Tuple(elems))
        } else {
            self.expect(&TokenKind::Punct(Punct::RParen))?;
            Ok(first)
        }
    }

    fn parse_record_pat_after_brace(&mut self) -> Result<Pat, ParseError> {
        let mut fields = Vec::new();
        if matches!(self.peek(), Some(TokenKind::Punct(Punct::RBrace))) {
            self.bump();
            return Ok(Pat::Record(fields));
        }
        loop {
            let name = self.expect_ident()?;
            let pat = if matches!(self.peek(), Some(TokenKind::Op(Op::Assign))) {
                self.bump();
                self.parse_pat()?
            } else {
                Pat::Var(name.clone())
            };
            fields.push((name, pat));
            if !matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                break;
            }
            self.bump();
            if matches!(self.peek(), Some(TokenKind::Punct(Punct::RBrace))) {
                break;
            }
        }
        self.expect(&TokenKind::Punct(Punct::RBrace))?;
        Ok(Pat::Record(fields))
    }

    fn parse_list_pat_after_bracket(&mut self) -> Result<Pat, ParseError> {
        let mut parts = Vec::new();
        if matches!(self.peek(), Some(TokenKind::Punct(Punct::RBracket))) {
            self.bump();
            return Ok(Pat::List(parts));
        }
        loop {
            if matches!(self.peek(), Some(TokenKind::Op(Op::DotDot))) {
                self.bump();
                let name = if let Some(TokenKind::Ident(_)) = self.peek() {
                    let n = self.expect_ident()?;
                    if n == "_" { None } else { Some(n) }
                } else {
                    None
                };
                parts.push(ListPart::Rest(name));
            } else {
                parts.push(ListPart::Pat(self.parse_pat()?));
            }
            if !matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                break;
            }
            self.bump();
            if matches!(self.peek(), Some(TokenKind::Punct(Punct::RBracket))) {
                break;
            }
        }
        self.expect(&TokenKind::Punct(Punct::RBracket))?;
        Ok(Pat::List(parts))
    }

    fn parse_pat_atom(&mut self) -> Result<Pat, ParseError> {
        match self.peek() {
            Some(TokenKind::Punct(Punct::LBracket)) => {
                self.bump();
                return self.parse_list_pat_after_bracket();
            }
            Some(TokenKind::Punct(Punct::LParen)) => {
                self.bump();
                return self.parse_paren_pat_after_lparen();
            }
            Some(TokenKind::Punct(Punct::LBrace)) => {
                self.bump();
                return self.parse_record_pat_after_brace();
            }
            _ => {}
        }
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
            other => Err(ParseError::new(format!("unexpected token in pattern: {other}"))),
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        match self.bump() {
            Some(TokenKind::Ident(name)) => Ok(name),
            Some(other) => Err(ParseError::new(format!("expected identifier, found {other}"))),
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
            if matches!(self.peek(), Some(TokenKind::Op(Op::Dot))) && FIELD_BP >= min_bp {
                self.bump();
                let name = self.expect_ident()?;
                lhs = Expr::Field(Box::new(lhs), name);
                continue;
            }
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
        let span = self.current_span();
        let tok = self
            .bump()
            .ok_or_else(|| ParseError::new("empty input").with_span(span.clone()))?;
        match tok {
            TokenKind::Int(n) => Ok(Expr::Lit(Lit::Int(n))),
            TokenKind::Float(f) => Ok(Expr::Lit(Lit::Float(f))),
            TokenKind::Str(s) => Ok(Expr::Lit(Lit::Str(s))),
            TokenKind::Bool(b) => Ok(Expr::Lit(Lit::Bool(b))),
            TokenKind::Ident(name) => Ok(Expr::Var(name)),
            TokenKind::Sym(name) => Ok(Expr::Sym(name)),
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
            TokenKind::Keyword(Keyword::Scope) => {
                self.expect(&TokenKind::Op(Op::Assign))?;
                let body = self.parse_body_after_block_intro()?;
                Ok(Expr::Scope(Box::new(body)))
            }
            TokenKind::Keyword(Keyword::Spawn) => {
                let inner = self.parse_expr_bp(0)?;
                Ok(Expr::Spawn(Box::new(inner)))
            }
            TokenKind::Keyword(Keyword::App) => {
                self.expect(&TokenKind::Op(Op::Assign))?;
                let body = self.parse_body_after_block_intro()?;
                Ok(Expr::AppBlock(Box::new(body)))
            }
            TokenKind::Keyword(Keyword::Match) => {
                let scrut = self.parse_expr_bp(0)?;
                self.expect(&TokenKind::Keyword(Keyword::With))?;
                self.skip_newlines();
                let mut arms = Vec::new();
                while matches!(self.peek(), Some(TokenKind::Punct(Punct::Bar))) {
                    self.bump();
                    let pat = self.parse_arm_pat()?;
                    let guard = if matches!(self.peek(), Some(TokenKind::Keyword(Keyword::When))) {
                        self.bump();
                        Some(self.parse_expr_bp(0)?)
                    } else {
                        None
                    };
                    self.expect(&TokenKind::Op(Op::RArrow))?;
                    let body = self.parse_expr_bp(0)?;
                    arms.push(MatchArm { pat, guard, body });
                    self.skip_newlines();
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
                let first = self.parse_expr_bp(0)?;
                if matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                    let mut elems = vec![first];
                    while matches!(self.peek(), Some(TokenKind::Punct(Punct::Comma))) {
                        self.bump();
                        if matches!(self.peek(), Some(TokenKind::Punct(Punct::RParen))) {
                            break;
                        }
                        elems.push(self.parse_expr_bp(0)?);
                    }
                    self.expect(&TokenKind::Punct(Punct::RParen))?;
                    Ok(Expr::Tuple(elems))
                } else {
                    self.expect(&TokenKind::Punct(Punct::RParen))?;
                    Ok(first)
                }
            }
            TokenKind::Punct(Punct::LBrace) => self.parse_record(),
            TokenKind::Punct(Punct::LBracket) => self.parse_series(),
            TokenKind::Punct(Punct::FrameOpen) => self.parse_dataframe(),
            TokenKind::Punct(Punct::ArrayOpen) => self.parse_array(),
            other => Err(ParseError::new(format!("unexpected token: {other}")).with_span(span)),
        }
    }
}

const APP_BP: u8 = 25;
const FIELD_BP: u8 = 28;

fn is_irrefutable_param_pat(pat: &Pat) -> bool {
    match pat {
        Pat::Wildcard | Pat::Var(_) => true,
        Pat::Lit(Lit::Unit) => true,
        Pat::Tuple(ps) => ps.iter().all(is_irrefutable_param_pat),
        Pat::Record(fs) => fs.iter().all(|(_, p)| is_irrefutable_param_pat(p)),
        Pat::As(inner, _) => is_irrefutable_param_pat(inner),
        _ => false,
    }
}

fn starts_with_uppercase(s: &str) -> bool {
    s.chars().next().is_some_and(|c| c.is_ascii_uppercase())
}

fn starts_type_atom(tok: &TokenKind) -> bool {
    matches!(
        tok,
        TokenKind::Ident(_)
            | TokenKind::Punct(
                Punct::Tick | Punct::LParen | Punct::LBrace | Punct::LBracket,
            )
    )
}

fn flatten_block(mut stmts: Vec<Stmt>) -> Expr {
    if stmts.len() == 1
        && matches!(stmts[0], Stmt::Expr(_))
        && let Some(Stmt::Expr(e)) = stmts.pop()
    {
        return e;
    }
    let trailing = if matches!(stmts.last(), Some(Stmt::Expr(_))) {
        if let Some(Stmt::Expr(e)) = stmts.pop() {
            Some(Box::new(e))
        } else {
            unreachable!()
        }
    } else {
        None
    };
    Expr::Block { stmts, trailing }
}

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
            | TokenKind::Punct(Punct::LParen | Punct::LBracket | Punct::LBrace)
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
