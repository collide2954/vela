//! Type checking and inference for the Vela language.

mod explain;
pub use explain::explain;

use std::collections::HashMap;
use vela_parser::{
    BinOp, Expr, ListPart, Lit, Pat, PostOp, Stmt, TypeDeclBody, UnOp, parse_expr, parse_program,
};

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
    Record(Vec<(String, Type)>, Option<Box<Type>>),
    DataFrame,
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    Named(String, Vec<Type>),
    Formula,
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
            Type::Record(fields, tail) => {
                let parts: Vec<String> =
                    fields.iter().map(|(n, t)| format!("{n}: {}", t.show())).collect();
                match tail {
                    None => format!("{{ {} }}", parts.join(", ")),
                    Some(t) => format!("{{ {} | {} }}", parts.join(", "), t.show()),
                }
            }
            Type::DataFrame => "DataFrame".into(),
            Type::Option(t) => format!("Option[{}]", t.show()),
            Type::Result(a, e) => format!("Result[{}, {}]", a.show(), e.show()),
            Type::Named(name, args) => {
                if args.is_empty() {
                    name.clone()
                } else {
                    let parts: Vec<String> = args.iter().map(|t| t.show()).collect();
                    format!("{name}[{}]", parts.join(", "))
                }
            }
            Type::Formula => "Formula".into(),
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
    pub code: &'static str,
}

impl TypeError {
    fn new(message: impl Into<String>) -> Self {
        Self { message: message.into(), code: "E0100" }
    }

    fn with_code(mut self, code: &'static str) -> Self {
        self.code = code;
        self
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

#[derive(Debug)]
struct Ctx {
    subst: HashMap<u32, Type>,
    fresh: u32,
    sums: HashMap<String, Vec<String>>,
    expected_return: Option<Type>,
}

impl Default for Ctx {
    fn default() -> Self {
        let mut sums = HashMap::new();
        sums.insert("Bool".into(), vec!["true".into(), "false".into()]);
        sums.insert("Option".into(), vec!["None".into(), "Some".into()]);
        sums.insert("Result".into(), vec!["Ok".into(), "Err".into()]);
        Self { subst: HashMap::new(), fresh: 0, sums, expected_return: None }
    }
}

impl Ctx {
    fn enter_function(&mut self, ret: Type) -> Option<Type> {
        self.expected_return.replace(ret)
    }

    fn exit_function(&mut self, saved: Option<Type>) {
        self.expected_return = saved;
    }
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
            Type::Record(fields, tail) => {
                let resolved_fields: Vec<(String, Type)> =
                    fields.iter().map(|(n, t)| (n.clone(), self.resolve(t))).collect();
                match tail {
                    None => Type::Record(resolved_fields, None),
                    Some(t) => {
                        let resolved_tail = self.resolve(t);
                        match resolved_tail {
                            Type::Record(more, more_tail) => {
                                let mut all = resolved_fields;
                                for (n, ty) in more {
                                    if !all.iter().any(|(name, _)| *name == n) {
                                        all.push((n, ty));
                                    }
                                }
                                Type::Record(all, more_tail)
                            }
                            other => Type::Record(resolved_fields, Some(Box::new(other))),
                        }
                    }
                }
            }
            Type::Option(t) => Type::Option(Box::new(self.resolve(t))),
            Type::Result(a, e) => {
                Type::Result(Box::new(self.resolve(a)), Box::new(self.resolve(e)))
            }
            Type::Named(name, args) => Type::Named(
                name.clone(),
                args.iter().map(|t| self.resolve(t)).collect(),
            ),
            other => other.clone(),
        }
    }

    fn unify_records(
        &mut self,
        fa: &[(String, Type)],
        ta: Option<&Type>,
        fb: &[(String, Type)],
        tb: Option<&Type>,
    ) -> Result<(), TypeError> {
        let mut a_only: Vec<(String, Type)> = Vec::new();
        let mut b_only: Vec<(String, Type)> = Vec::new();
        for (na, t) in fa {
            if let Some((_, tb)) = fb.iter().find(|(nb, _)| nb == na) {
                self.unify(t, tb)?;
            } else {
                a_only.push((na.clone(), t.clone()));
            }
        }
        for (nb, t) in fb {
            if !fa.iter().any(|(na, _)| na == nb) {
                b_only.push((nb.clone(), t.clone()));
            }
        }
        match (ta, tb) {
            (None, None) => {
                if !a_only.is_empty() {
                    return Err(TypeError::new(format!(
                        "record missing fields: {:?}",
                        a_only.iter().map(|(n, _)| n).collect::<Vec<_>>()
                    )));
                }
                if !b_only.is_empty() {
                    return Err(TypeError::new(format!(
                        "record has extra fields: {:?}",
                        b_only.iter().map(|(n, _)| n).collect::<Vec<_>>()
                    )));
                }
                Ok(())
            }
            (Some(t_a), None) => {
                if !a_only.is_empty() {
                    return Err(TypeError::new(format!(
                        "record missing fields: {:?}",
                        a_only.iter().map(|(n, _)| n).collect::<Vec<_>>()
                    )));
                }
                let extension = Type::Record(b_only, None);
                self.unify(t_a, &extension)
            }
            (None, Some(t_b)) => {
                if !b_only.is_empty() {
                    return Err(TypeError::new(format!(
                        "record has extra fields: {:?}",
                        b_only.iter().map(|(n, _)| n).collect::<Vec<_>>()
                    )));
                }
                let extension = Type::Record(a_only, None);
                self.unify(t_b, &extension)
            }
            (Some(t_a), Some(t_b)) => {
                let rho = self.fresh_var();
                let a_ext = Type::Record(b_only, Some(Box::new(rho.clone())));
                let b_ext = Type::Record(a_only, Some(Box::new(rho)));
                self.unify(t_a, &a_ext)?;
                self.unify(t_b, &b_ext)?;
                Ok(())
            }
        }
    }

    fn instantiate(&mut self, scheme: &Scheme) -> Type {
        let mut subst: HashMap<u32, Type> = HashMap::new();
        for &v in &scheme.vars {
            subst.insert(v, self.fresh_var());
        }
        apply_subst(&scheme.ty, &subst)
    }

    fn fresh_id(&mut self) -> u32 {
        let n = self.fresh;
        self.fresh += 1;
        n
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
            Type::Record(fields, tail) => {
                fields.iter().any(|(_, t)| self.occurs(n, t))
                    || tail.as_ref().is_some_and(|t| self.occurs(n, t))
            }
            Type::Option(t) => self.occurs(n, &t),
            Type::Result(a, e) => self.occurs(n, &a) || self.occurs(n, &e),
            Type::Named(_, args) => args.iter().any(|t| self.occurs(n, t)),
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
            (Type::Named(n1, a1), Type::Named(n2, a2)) => {
                if n1 != n2 {
                    return Err(TypeError::new(format!(
                        "cannot unify type `{n1}` with `{n2}`"
                    )));
                }
                if a1.len() != a2.len() {
                    return Err(TypeError::new(format!(
                        "type `{n1}` arity mismatch: {} vs {}",
                        a1.len(),
                        a2.len()
                    )));
                }
                for (x, y) in a1.iter().zip(a2.iter()) {
                    self.unify(x, y)?;
                }
                Ok(())
            }
            (Type::Record(fa, ta), Type::Record(fb, tb)) => {
                self.unify_records(&fa, ta.as_deref(), &fb, tb.as_deref())
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
    let env = prelude(&mut ctx);
    let t = infer(&expr, &env, &mut ctx)?;
    Ok(ctx.resolve(&t))
}

pub fn check_program(src: &str) -> Result<Type, TypeError> {
    let program = parse_program(src)
        .map_err(|e| TypeError::new(format!("parse error: {}", e.message)))?;
    let mut ctx = Ctx::default();
    let mut env = prelude(&mut ctx);
    let mut last = Type::Unit;
    for stmt in &program.stmts {
        last = check_stmt(stmt, &mut env, &mut ctx)?;
    }
    Ok(ctx.resolve(&last))
}

fn prelude(ctx: &mut Ctx) -> Env {
    let mut env = Env::new();

    let series_of = |t: Type| Type::Series(Box::new(t));
    let fn_of = |a: Type, b: Type| Type::Fn(Box::new(a), Box::new(b));

    // length : [a] -> Int
    {
        let a = ctx.fresh_id();
        env = env.extend(
            "length".into(),
            Scheme {
                vars: vec![a],
                ty: fn_of(series_of(Type::Var(a)), Type::Int),
            },
        );
    }
    // map : (a -> b) -> [a] -> [b]
    {
        let a = ctx.fresh_id();
        let b = ctx.fresh_id();
        env = env.extend(
            "map".into(),
            Scheme {
                vars: vec![a, b],
                ty: fn_of(
                    fn_of(Type::Var(a), Type::Var(b)),
                    fn_of(series_of(Type::Var(a)), series_of(Type::Var(b))),
                ),
            },
        );
    }
    // filter : (a -> Bool) -> [a] -> [a]
    {
        let a = ctx.fresh_id();
        env = env.extend(
            "filter".into(),
            Scheme {
                vars: vec![a],
                ty: fn_of(
                    fn_of(Type::Var(a), Type::Bool),
                    fn_of(series_of(Type::Var(a)), series_of(Type::Var(a))),
                ),
            },
        );
    }
    // sum : [Int] -> Int
    env = env.extend(
        "sum".into(),
        Scheme { vars: Vec::new(), ty: fn_of(series_of(Type::Int), Type::Int) },
    );
    // println : a -> ()
    {
        let a = ctx.fresh_id();
        env = env.extend(
            "println".into(),
            Scheme { vars: vec![a], ty: fn_of(Type::Var(a), Type::Unit) },
        );
    }
    // print : a -> ()
    {
        let a = ctx.fresh_id();
        env = env.extend(
            "print".into(),
            Scheme { vars: vec![a], ty: fn_of(Type::Var(a), Type::Unit) },
        );
    }
    // read_file : String -> Result<String, IoError>
    env = env.extend(
        "read_file".into(),
        Scheme {
            vars: Vec::new(),
            ty: fn_of(
                Type::String,
                Type::Result(
                    Box::new(Type::String),
                    Box::new(Type::Named("IoError".into(), Vec::new())),
                ),
            ),
        },
    );

    // Float namespace
    env = env.extend(
        "Float".into(),
        Scheme {
            vars: Vec::new(),
            ty: Type::Record(
                vec![
                    ("of_int".into(), fn_of(Type::Int, Type::Float)),
                    ("to_string".into(), fn_of(Type::Float, Type::String)),
                ],
                None,
            ),
        },
    );

    // Int namespace
    env = env.extend(
        "Int".into(),
        Scheme {
            vars: Vec::new(),
            ty: Type::Record(
                vec![
                    ("of_float".into(), fn_of(Type::Float, Type::Int)),
                    ("to_string".into(), fn_of(Type::Int, Type::String)),
                ],
                None,
            ),
        },
    );

    // Result namespace
    {
        let a = ctx.fresh_id();
        let e = ctx.fresh_id();
        env = env.extend(
            "Result".into(),
            Scheme {
                vars: vec![a, e],
                ty: Type::Record(
                    vec![("unwrap".into(), fn_of(
                        Type::Result(Box::new(Type::Var(a)), Box::new(Type::Var(e))),
                        Type::Var(a),
                    ))],
                    None,
                ),
            },
        );
    }

    // Option namespace
    {
        let a = ctx.fresh_id();
        env = env.extend(
            "Option".into(),
            Scheme {
                vars: vec![a],
                ty: Type::Record(
                    vec![("unwrap".into(), fn_of(
                        Type::Option(Box::new(Type::Var(a))),
                        Type::Var(a),
                    ))],
                    None,
                ),
            },
        );
    }

    // String namespace
    env = env.extend(
        "String".into(),
        Scheme {
            vars: Vec::new(),
            ty: Type::Record(
                vec![
                    ("length".into(), fn_of(Type::String, Type::Int)),
                    (
                        "concat".into(),
                        fn_of(series_of(Type::String), Type::String),
                    ),
                ],
                None,
            ),
        },
    );

    // Stream namespace
    {
        let s = ctx.fresh_id();
        let a = ctx.fresh_id();
        env = env.extend(
            "Stream".into(),
            Scheme {
                vars: vec![s, a],
                ty: Type::Record(
                    vec![("unfold".into(), fn_of(
                        fn_of(
                            Type::Var(s),
                            Type::Option(Box::new(Type::Tuple(vec![
                                Type::Var(a),
                                Type::Var(s),
                            ]))),
                        ),
                        fn_of(Type::Var(s), series_of(Type::Var(a))),
                    ))],
                    None,
                ),
            },
        );
    }

    // Stats stubs typed for Float series
    env = env.extend(
        "mean".into(),
        Scheme {
            vars: Vec::new(),
            ty: fn_of(series_of(Type::Float), Type::Float),
        },
    );
    env = env.extend(
        "std".into(),
        Scheme {
            vars: Vec::new(),
            ty: fn_of(series_of(Type::Float), Type::Float),
        },
    );
    env = env.extend(
        "min".into(),
        Scheme {
            vars: Vec::new(),
            ty: fn_of(series_of(Type::Float), Type::Float),
        },
    );
    env = env.extend(
        "max".into(),
        Scheme {
            vars: Vec::new(),
            ty: fn_of(series_of(Type::Float), Type::Float),
        },
    );

    {
        let a = ctx.fresh_id();
        env = env.extend(
            "None".into(),
            Scheme { vars: vec![a], ty: Type::Option(Box::new(Type::Var(a))) },
        );
    }
    {
        let a = ctx.fresh_id();
        env = env.extend(
            "Some".into(),
            Scheme {
                vars: vec![a],
                ty: Type::Fn(
                    Box::new(Type::Var(a)),
                    Box::new(Type::Option(Box::new(Type::Var(a)))),
                ),
            },
        );
    }
    {
        let a = ctx.fresh_id();
        let e = ctx.fresh_id();
        env = env.extend(
            "Ok".into(),
            Scheme {
                vars: vec![a, e],
                ty: Type::Fn(
                    Box::new(Type::Var(a)),
                    Box::new(Type::Result(Box::new(Type::Var(a)), Box::new(Type::Var(e)))),
                ),
            },
        );
    }
    {
        let a = ctx.fresh_id();
        let e = ctx.fresh_id();
        env = env.extend(
            "Err".into(),
            Scheme {
                vars: vec![a, e],
                ty: Type::Fn(
                    Box::new(Type::Var(e)),
                    Box::new(Type::Result(Box::new(Type::Var(a)), Box::new(Type::Var(e)))),
                ),
            },
        );
    }
    env
}

fn check_stmt(stmt: &Stmt, env: &mut Env, ctx: &mut Ctx) -> Result<Type, TypeError> {
    match stmt {
        Stmt::Let { name, params, return_ty, body, recursive } => {
            let mut translator = TyTranslator::new();
            let mut inner_env = env.clone();
            if *recursive {
                let placeholder = ctx.fresh_var();
                inner_env =
                    inner_env.extend(name.clone(), Scheme::mono(placeholder));
            }
            let mut param_types = Vec::with_capacity(params.len());
            for p in params {
                let pt = match &p.ty {
                    Some(ty) => translator.translate(ty, ctx)?,
                    None => ctx.fresh_var(),
                };
                let (pat_ty, bindings) = infer_pat(&p.pat, &inner_env, ctx)?;
                ctx.unify(&pt, &pat_ty)?;
                for (n, t) in bindings {
                    inner_env = inner_env.extend(n, Scheme::mono(t));
                }
                param_types.push(pt);
            }
            let body_ty = if params.is_empty() {
                infer(body, &inner_env, ctx)?
            } else {
                let return_var = match return_ty {
                    Some(rt) => translator.translate(rt, ctx)?,
                    None => ctx.fresh_var(),
                };
                let saved = ctx.enter_function(return_var.clone());
                let bt = infer(body, &inner_env, ctx)?;
                ctx.unify(&return_var, &bt)?;
                ctx.exit_function(saved);
                ctx.resolve(&return_var)
            };
            if params.is_empty()
                && let Some(rt) = return_ty
            {
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
        Stmt::LetRecGroup(bindings) => {
            let mut inner_env = env.clone();
            let mut placeholders = Vec::with_capacity(bindings.len());
            for b in bindings {
                let placeholder = ctx.fresh_var();
                placeholders.push(placeholder.clone());
                inner_env = inner_env.extend(b.name.clone(), Scheme::mono(placeholder));
            }
            let mut binding_types = Vec::with_capacity(bindings.len());
            for (b, placeholder) in bindings.iter().zip(placeholders.iter()) {
                let mut translator = TyTranslator::new();
                let mut body_env = inner_env.clone();
                let mut param_types = Vec::with_capacity(b.params.len());
                for p in &b.params {
                    let pt = match &p.ty {
                        Some(ty) => translator.translate(ty, ctx)?,
                        None => ctx.fresh_var(),
                    };
                    let (pat_ty, pat_bindings) = infer_pat(&p.pat, &body_env, ctx)?;
                    ctx.unify(&pt, &pat_ty)?;
                    for (n, t) in pat_bindings {
                        body_env = body_env.extend(n, Scheme::mono(t));
                    }
                    param_types.push(pt);
                }
                let body_ty = if b.params.is_empty() {
                    infer(&b.body, &body_env, ctx)?
                } else {
                    let return_var = match &b.return_ty {
                        Some(rt) => translator.translate(rt, ctx)?,
                        None => ctx.fresh_var(),
                    };
                    let saved = ctx.enter_function(return_var.clone());
                    let bt = infer(&b.body, &body_env, ctx)?;
                    ctx.unify(&return_var, &bt)?;
                    ctx.exit_function(saved);
                    ctx.resolve(&return_var)
                };
                if b.params.is_empty()
                    && let Some(rt) = &b.return_ty
                {
                    let rt_translated = translator.translate(rt, ctx)?;
                    ctx.unify(&body_ty, &rt_translated)?;
                }
                let ty = param_types.into_iter().rev().fold(body_ty, |acc, pt| {
                    Type::Fn(Box::new(pt), Box::new(acc))
                });
                ctx.unify(placeholder, &ty)?;
                binding_types.push(ty);
            }
            for (b, ty) in bindings.iter().zip(binding_types.iter()) {
                let scheme = ctx.generalize(env, ty);
                *env = env.extend(b.name.clone(), scheme);
            }
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
        Stmt::TypeDecl(decl) => {
            let mut translator = TyTranslator::new();
            let mut param_vars: Vec<u32> = Vec::new();
            for param_name in &decl.params {
                let v = ctx.fresh_var();
                if let Type::Var(n) = v {
                    translator.named_vars.insert(param_name.clone(), Type::Var(n));
                    param_vars.push(n);
                }
            }
            let result_type = Type::Named(
                decl.name.clone(),
                decl.params
                    .iter()
                    .map(|p| translator.named_vars[p].clone())
                    .collect(),
            );
            match &decl.body {
                TypeDeclBody::Sum(variants) => {
                    let mut variant_names = Vec::with_capacity(variants.len());
                    for v in variants {
                        variant_names.push(v.name.clone());
                        let mut ty = result_type.clone();
                        for arg in v.args.iter().rev() {
                            let arg_ty = translator.translate(arg, ctx)?;
                            ty = Type::Fn(Box::new(arg_ty), Box::new(ty));
                        }
                        let scheme = Scheme { vars: param_vars.clone(), ty };
                        *env = env.extend(v.name.clone(), scheme);
                    }
                    ctx.sums.insert(decl.name.clone(), variant_names);
                }
                TypeDeclBody::Alias(_) => {}
            }
            Ok(Type::Unit)
        }
        Stmt::Mutate { name, body } => {
            let var_scheme = env
                .lookup(name)
                .cloned()
                .ok_or_else(|| TypeError::new(format!("unbound name: {name}")))?;
            let var_ty = ctx.instantiate(&var_scheme);
            let body_ty = infer(body, env, ctx)?;
            ctx.unify(&var_ty, &body_ty)?;
            Ok(Type::Unit)
        }
        Stmt::For { binding, iter, body } => {
            let iter_ty = infer(iter, env, ctx)?;
            let elem_ty = ctx.fresh_var();
            ctx.unify(&iter_ty, &Type::Series(Box::new(elem_ty.clone())))?;
            let (pat_ty, bindings) = infer_pat(binding, env, ctx)?;
            ctx.unify(&elem_ty, &pat_ty)?;
            let mut body_env = env.clone();
            for (n, t) in bindings {
                body_env = body_env.extend(n, Scheme::mono(t));
            }
            let body_ty = infer(body, &body_env, ctx)?;
            ctx.unify(&body_ty, &Type::Unit)?;
            Ok(Type::Unit)
        }
        Stmt::Destructure { pat, body } => {
            let body_ty = infer(body, env, ctx)?;
            let (pat_ty, bindings) = infer_pat(pat, env, ctx)?;
            ctx.unify(&body_ty, &pat_ty)?;
            for (n, t) in bindings {
                let resolved = ctx.resolve(&t);
                *env = env.extend(n, Scheme::mono(resolved));
            }
            Ok(Type::Unit)
        }
        Stmt::Expr(e) => infer(e, env, ctx),
        Stmt::TraitDecl(decl) => {
            let mut translator = TyTranslator::new();
            let tv_id = ctx.fresh_id();
            translator
                .named_vars
                .insert(decl.type_var.clone(), Type::Var(tv_id));
            for method in &decl.methods {
                let return_ty = translator.translate(&method.return_ty, ctx)?;
                let mut ty = return_ty;
                for p in method.params.iter().rev() {
                    let pt = match &p.ty {
                        Some(t) => translator.translate(t, ctx)?,
                        None => {
                            return Err(TypeError::new(
                                "trait method parameters must be annotated",
                            ));
                        }
                    };
                    ty = Type::Fn(Box::new(pt), Box::new(ty));
                }
                let scheme = Scheme { vars: vec![tv_id], ty };
                *env = env.extend(method.name.clone(), scheme);
            }
            Ok(Type::Unit)
        }
        Stmt::Impl(block) => {
            let mut translator = TyTranslator::new();
            let target_ty = translator.translate(&block.ty, ctx)?;
            for method in &block.methods {
                let mut inner_env = env.clone();
                let mut param_types = Vec::with_capacity(method.params.len());
                for p in &method.params {
                    let pt = match &p.ty {
                        Some(t) => translator.translate(t, ctx)?,
                        None => ctx.fresh_var(),
                    };
                    let (pat_ty, bindings) = infer_pat(&p.pat, &inner_env, ctx)?;
                    ctx.unify(&pt, &pat_ty)?;
                    for (n, t) in bindings {
                        inner_env = inner_env.extend(n, Scheme::mono(t));
                    }
                    param_types.push(pt);
                }
                let return_var = match &method.return_ty {
                    Some(rt) => translator.translate(rt, ctx)?,
                    None => ctx.fresh_var(),
                };
                let saved = ctx.enter_function(return_var.clone());
                let bt = infer(&method.body, &inner_env, ctx)?;
                ctx.unify(&return_var, &bt)?;
                ctx.exit_function(saved);
                let _ = (target_ty.clone(), param_types);
            }
            Ok(Type::Unit)
        }
        Stmt::Tests(_)
        | Stmt::Extern { .. }
        | Stmt::Import { .. } => Ok(Type::Unit),
        Stmt::Input { name, body } | Stmt::Output { name, body } => {
            let ty = infer(body, env, ctx)?;
            let ty = ctx.resolve(&ty);
            *env = env.extend(name.clone(), Scheme::mono(ty));
            Ok(Type::Unit)
        }
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
        let pt = match &p.ty {
            Some(ty) => {
                let mut tr = TyTranslator::new();
                tr.translate(ty, ctx)?
            }
            None => ctx.fresh_var(),
        };
        let (pat_ty, bindings) = infer_pat(&p.pat, &env, ctx)?;
        ctx.unify(&pt, &pat_ty)?;
        for (n, t) in bindings {
            env = env.extend(n, Scheme::mono(t));
        }
        param_types.push(pt);
    }
    let return_ty = ctx.fresh_var();
    let saved = ctx.enter_function(return_ty.clone());
    let body_ty = infer(body, &env, ctx)?;
    ctx.unify(&return_ty, &body_ty)?;
    ctx.exit_function(saved);
    Ok(param_types
        .into_iter()
        .rev()
        .fold(return_ty, |acc, pt| Type::Fn(Box::new(pt), Box::new(acc))))
}

fn infer(expr: &Expr, env: &Env, ctx: &mut Ctx) -> Result<Type, TypeError> {
    match expr {
        Expr::Lit(Lit::Int(_)) => Ok(Type::Int),
        Expr::Lit(Lit::Float(_)) => Ok(Type::Float),
        Expr::Lit(Lit::Str(_)) => Ok(Type::String),
        Expr::Lit(Lit::Bool(_)) => Ok(Type::Bool),
        Expr::Lit(Lit::Unit) => Ok(Type::Unit),
        Expr::Var(name) => {
            let scheme = env.lookup(name).cloned().ok_or_else(|| {
                TypeError::new(format!("unbound name: {name}")).with_code("E0110")
            })?;
            Ok(ctx.instantiate(&scheme))
        }
        Expr::UnaryOp(op, inner) => infer_unary(*op, inner, env, ctx),
        Expr::Postfix(PostOp::Question, inner) => {
            let t = infer(inner, env, ctx)?;
            let a = ctx.fresh_var();
            let e = ctx.fresh_var();
            let expected = Type::Result(Box::new(a.clone()), Box::new(e.clone()));
            ctx.unify(&t, &expected)?;
            let return_ty = ctx.expected_return.clone().ok_or_else(|| {
                TypeError::new(
                    "`?` requires the enclosing function to return a Result",
                )
            })?;
            let outer_ok = ctx.fresh_var();
            let expected_return =
                Type::Result(Box::new(outer_ok), Box::new(e.clone()));
            ctx.unify(&return_ty, &expected_return)?;
            Ok(ctx.resolve(&a))
        }
        Expr::BinOp(BinOp::Tilde, _, _) => Ok(Type::Formula),
        Expr::BinOp(op, lhs, rhs) => infer_binary(*op, lhs, rhs, env, ctx),
        Expr::Lambda(params, body) => lambda_type(params, body, env, ctx),
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
            Ok(Type::Record(fts, None))
        }
        Expr::RecordUpdate(base, updates) => {
            let base_ty = infer(base, env, ctx)?;
            let base_ty = ctx.resolve(&base_ty);
            let Type::Record(mut fields, tail) = base_ty else {
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
            Ok(Type::Record(fields, tail))
        }
        Expr::Field(target, name) => {
            let target_ty = infer(target, env, ctx)?;
            let resolved = ctx.resolve(&target_ty);
            if matches!(resolved, Type::DataFrame) {
                let inner = ctx.fresh_var();
                return Ok(Type::Series(Box::new(Type::Option(Box::new(inner)))));
            }
            let field_ty = ctx.fresh_var();
            let row_tail = ctx.fresh_var();
            let expected = Type::Record(
                vec![(name.clone(), field_ty.clone())],
                Some(Box::new(row_tail)),
            );
            ctx.unify(&target_ty, &expected)?;
            Ok(ctx.resolve(&field_ty))
        }
        Expr::Match(scrut, arms) => {
            let s_ty = infer(scrut, env, ctx)?;
            let result_ty = ctx.fresh_var();
            for arm in arms {
                let (pat_ty, bindings) = infer_pat(&arm.pat, env, ctx)?;
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
            check_exhaustive(&s_ty, arms, ctx).map_err(|e| e.with_code("E0130"))?;
            Ok(ctx.resolve(&result_ty))
        }
        Expr::ArrayLit(rows) => {
            let inner = ctx.fresh_var();
            for row in rows {
                for e in row {
                    let t = infer(e, env, ctx)?;
                    ctx.unify(&inner, &t)?;
                }
            }
            Ok(Type::Series(Box::new(ctx.resolve(&inner))))
        }
        Expr::DataFrameLit(cols) => {
            for (_, e) in cols {
                let t = infer(e, env, ctx)?;
                let inner = ctx.fresh_var();
                ctx.unify(&t, &Type::Series(Box::new(inner)))?;
            }
            Ok(Type::DataFrame)
        }
        Expr::Scope(body) => {
            let _ = infer(body, env, ctx)?;
            Ok(Type::Unit)
        }
        Expr::Spawn(inner) => {
            let _ = infer(inner, env, ctx)?;
            Ok(Type::Unit)
        }
        Expr::AppBlock(body) => {
            let _ = infer(body, env, ctx)?;
            Ok(Type::Unit)
        }
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
            vela_parser::Ty::Con(name) => self.translate_con(name, &[], ctx),
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
                Ok(Type::Record(translated, None))
            }
            vela_parser::Ty::App(base, args) => {
                let name = match base.as_ref() {
                    vela_parser::Ty::Con(n) => n.clone(),
                    other => {
                        return Err(TypeError::new(format!(
                            "type application base must be a name, got {other:?}"
                        )));
                    }
                };
                self.translate_con(&name, args, ctx)
            }
        }
    }

    fn translate_con(
        &mut self,
        name: &str,
        args: &[vela_parser::Ty],
        ctx: &mut Ctx,
    ) -> Result<Type, TypeError> {
        if args.is_empty()
            && let Some(t) = self.named_vars.get(name)
        {
            return Ok(t.clone());
        }
        let translated_args: Result<Vec<Type>, TypeError> = args
            .iter()
            .filter(|a| !matches!(a, vela_parser::Ty::Con(n) if n == "_dim"))
            .map(|a| self.translate(a, ctx))
            .collect();
        let targs = translated_args?;
        match name {
            "Int" if targs.is_empty() => Ok(Type::Int),
            "UInt" if targs.is_empty() => Ok(Type::UInt),
            "BigInt" if targs.is_empty() => Ok(Type::BigInt),
            "Float" if targs.is_empty() => Ok(Type::Float),
            "Decimal" if targs.is_empty() => Ok(Type::Decimal),
            "Bool" if targs.is_empty() => Ok(Type::Bool),
            "String" if targs.is_empty() => Ok(Type::String),
            "Symbol" if targs.is_empty() => Ok(Type::Symbol),
            "DataFrame" if targs.is_empty() => Ok(Type::DataFrame),
            "Option" if targs.len() == 1 => Ok(Type::Option(Box::new(targs.into_iter().next().expect("len 1")))),
            "Result" if targs.len() == 2 => {
                let mut it = targs.into_iter();
                let a = it.next().expect("a");
                let e = it.next().expect("e");
                Ok(Type::Result(Box::new(a), Box::new(e)))
            }
            "Array" if !targs.is_empty() => {
                Ok(Type::Series(Box::new(targs.into_iter().next().expect("a"))))
            }
            other => Ok(Type::Named(other.into(), targs)),
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
        Type::Record(fs, tail) => Type::Record(
            fs.iter().map(|(n, t)| (n.clone(), apply_subst(t, subst))).collect(),
            tail.as_ref().map(|t| Box::new(apply_subst(t, subst))),
        ),
        Type::Option(t) => Type::Option(Box::new(apply_subst(t, subst))),
        Type::Result(a, e) => {
            Type::Result(Box::new(apply_subst(a, subst)), Box::new(apply_subst(e, subst)))
        }
        Type::Named(name, args) => Type::Named(
            name.clone(),
            args.iter().map(|t| apply_subst(t, subst)).collect(),
        ),
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
        Type::Record(fs, tail) => {
            for (_, t) in &fs {
                collect_ftv(t, ctx, out);
            }
            if let Some(t) = tail.as_deref() {
                collect_ftv(t, ctx, out);
            }
        }
        Type::Option(t) => collect_ftv(&t, ctx, out),
        Type::Result(a, e) => {
            collect_ftv(&a, ctx, out);
            collect_ftv(&e, ctx, out);
        }
        Type::Named(_, args) => {
            for t in &args {
                collect_ftv(t, ctx, out);
            }
        }
        _ => {}
    }
}

fn check_exhaustive(
    scrut_ty: &Type,
    arms: &[vela_parser::MatchArm],
    ctx: &Ctx,
) -> Result<(), TypeError> {
    let resolved = ctx.resolve(scrut_ty);
    for arm in arms {
        if arm.guard.is_none() && is_absorbing(&arm.pat) {
            return Ok(());
        }
    }
    let required: Vec<String> = match &resolved {
        Type::Bool => vec!["true".into(), "false".into()],
        Type::Option(_) => vec!["None".into(), "Some".into()],
        Type::Result(_, _) => vec!["Ok".into(), "Err".into()],
        Type::Named(name, _) => match ctx.sums.get(name) {
            Some(vs) => vs.clone(),
            None => {
                return Err(TypeError::new(format!(
                    "non-exhaustive match: type {name} has no known variants"
                )));
            }
        },
        other => {
            return Err(TypeError::new(format!(
                "non-exhaustive match on {} (requires a wildcard arm)",
                other.show()
            )));
        }
    };
    let mut covered = std::collections::BTreeSet::new();
    for arm in arms {
        if arm.guard.is_some() {
            continue;
        }
        collect_covered(&arm.pat, &mut covered);
    }
    let missing: Vec<&String> = required.iter().filter(|c| !covered.contains(*c)).collect();
    if !missing.is_empty() {
        let names: Vec<String> = missing.into_iter().cloned().collect();
        return Err(TypeError::new(format!(
            "non-exhaustive match - missing: {}",
            names.join(", ")
        )));
    }
    Ok(())
}

fn is_absorbing(pat: &Pat) -> bool {
    match pat {
        Pat::Wildcard | Pat::Var(_) => true,
        Pat::As(inner, _) => is_absorbing(inner),
        Pat::Or(alts) => alts.iter().any(is_absorbing),
        Pat::Tuple(pats) => pats.iter().all(is_absorbing),
        Pat::Record(fields) => fields.iter().all(|(_, p)| is_absorbing(p)),
        Pat::List(parts) => {
            parts.len() == 1 && matches!(parts[0], ListPart::Rest(_))
        }
        _ => false,
    }
}

fn collect_covered(pat: &Pat, out: &mut std::collections::BTreeSet<String>) {
    match pat {
        Pat::Lit(Lit::Bool(true)) => {
            out.insert("true".into());
        }
        Pat::Lit(Lit::Bool(false)) => {
            out.insert("false".into());
        }
        Pat::Cons(name, _) => {
            out.insert(name.clone());
        }
        Pat::As(inner, _) => collect_covered(inner, out),
        Pat::Or(alts) => {
            for p in alts {
                collect_covered(p, out);
            }
        }
        _ => {}
    }
}

fn infer_pat(
    pat: &Pat,
    env: &Env,
    ctx: &mut Ctx,
) -> Result<(Type, Vec<(String, Type)>), TypeError> {
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
            let (t, mut bs) = infer_pat(inner, env, ctx)?;
            bs.push((name.clone(), t.clone()));
            Ok((t, bs))
        }
        Pat::Or(alts) => {
            if alts.is_empty() {
                return Err(TypeError::new("empty or-pattern"));
            }
            let (t, bs) = infer_pat(&alts[0], env, ctx)?;
            for a in &alts[1..] {
                let (t2, _) = infer_pat(a, env, ctx)?;
                ctx.unify(&t, &t2)?;
            }
            Ok((t, bs))
        }
        Pat::Cons(name, args) => {
            let scheme = env
                .lookup(name)
                .cloned()
                .ok_or_else(|| TypeError::new(format!("unbound constructor: {name}")))?;
            let mut current = ctx.instantiate(&scheme);
            let mut bindings = Vec::new();
            for arg in args {
                let (arg_ty, mut arg_bs) = infer_pat(arg, env, ctx)?;
                let result = ctx.fresh_var();
                let expected = Type::Fn(Box::new(arg_ty), Box::new(result.clone()));
                ctx.unify(&current, &expected)?;
                current = ctx.resolve(&result);
                bindings.append(&mut arg_bs);
            }
            Ok((current, bindings))
        }
        Pat::Tuple(pats) => {
            let mut types = Vec::with_capacity(pats.len());
            let mut bindings = Vec::new();
            for p in pats {
                let (pt, mut bs) = infer_pat(p, env, ctx)?;
                types.push(pt);
                bindings.append(&mut bs);
            }
            Ok((Type::Tuple(types), bindings))
        }
        Pat::Record(fields) => {
            let mut pat_fields = Vec::with_capacity(fields.len());
            let mut bindings = Vec::new();
            for (n, p) in fields {
                let (pt, mut bs) = infer_pat(p, env, ctx)?;
                pat_fields.push((n.clone(), pt));
                bindings.append(&mut bs);
            }
            let tail = ctx.fresh_var();
            Ok((Type::Record(pat_fields, Some(Box::new(tail))), bindings))
        }
        Pat::List(parts) => {
            let elem_ty = ctx.fresh_var();
            let mut bindings = Vec::new();
            for part in parts {
                match part {
                    ListPart::Pat(p) => {
                        let (pt, mut bs) = infer_pat(p, env, ctx)?;
                        ctx.unify(&elem_ty, &pt)?;
                        bindings.append(&mut bs);
                    }
                    ListPart::Rest(Some(name)) => {
                        bindings.push((
                            name.clone(),
                            Type::Series(Box::new(elem_ty.clone())),
                        ));
                    }
                    ListPart::Rest(None) => {}
                }
            }
            Ok((Type::Series(Box::new(elem_ty)), bindings))
        }
        Pat::Range(lo, hi) => {
            let (lo_t, _) = infer_pat(lo, env, ctx)?;
            let (hi_t, _) = infer_pat(hi, env, ctx)?;
            ctx.unify(&lo_t, &hi_t)?;
            Ok((lo_t, Vec::new()))
        }
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
        BinOp::Tilde => Ok(Type::Formula),
    }
}
