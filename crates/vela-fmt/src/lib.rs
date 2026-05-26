//! Pretty-printer for the Vela language. No options, no configuration.

use vela_parser::{
    BinOp, Expr, ImplBlock, ImplMethod, ImportKind, LetBinding, ListPart, Lit, MatchArm, Param,
    Pat, PostOp, Program, Stmt, TestCase, TraitDecl, TraitMethodSig, Ty, TypeDecl, TypeDeclBody,
    TypeVariant, UnOp, parse_program,
};

const INDENT: &str = "    ";

pub fn format_source(src: &str) -> Result<String, String> {
    let program = parse_program(src).map_err(|e| e.message)?;
    Ok(format_program(&program))
}

pub fn format_program(program: &Program) -> String {
    let mut p = Printer::new();
    p.program(program);
    p.out
}

struct Printer {
    out: String,
    indent: usize,
}

impl Printer {
    fn new() -> Self {
        Self {
            out: String::new(),
            indent: 0,
        }
    }

    fn push(&mut self, s: &str) {
        self.out.push_str(s);
    }

    fn newline(&mut self) {
        self.out.push('\n');
        for _ in 0..self.indent {
            self.out.push_str(INDENT);
        }
    }

    fn with_indent<F: FnOnce(&mut Self)>(&mut self, f: F) {
        self.indent += 1;
        f(self);
        self.indent -= 1;
    }

    fn program(&mut self, p: &Program) {
        let (imports, rest): (Vec<_>, Vec<_>) = p
            .stmts
            .iter()
            .partition(|s| matches!(s, Stmt::Import { .. }));
        let mut sorted_imports = imports.clone();
        sorted_imports.sort_by(|a, b| import_sort_key(a).cmp(&import_sort_key(b)));
        let mut first = true;
        let mut prev_group: Option<u8> = None;
        for stmt in &sorted_imports {
            let group = import_group(stmt);
            if !first && prev_group.is_some_and(|g| g != group) {
                self.push("\n");
            }
            if !first {
                self.push("\n");
            }
            self.stmt(stmt);
            first = false;
            prev_group = Some(group);
        }
        if !sorted_imports.is_empty() && !rest.is_empty() {
            self.push("\n\n");
        }
        for (i, stmt) in rest.iter().enumerate() {
            if i > 0 {
                self.push("\n\n");
            }
            self.stmt(stmt);
        }
        if !p.stmts.is_empty() {
            self.push("\n");
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Let {
                name,
                params,
                return_ty,
                body,
                recursive,
            } => {
                self.push("let ");
                if *recursive {
                    self.push("rec ");
                }
                self.push(name);
                for p in params {
                    self.push(" ");
                    self.param(p);
                }
                if let Some(rt) = return_ty {
                    self.push(" : ");
                    self.ty(rt);
                }
                self.push(" = ");
                self.body(body);
            }
            Stmt::LetRecGroup(bindings) => {
                if let Some((first, rest)) = bindings.split_first() {
                    self.push("let rec ");
                    self.let_binding_tail(first);
                    for b in rest {
                        self.newline();
                        self.push("and ");
                        self.let_binding_tail(b);
                    }
                }
            }
            Stmt::Var { name, ty, body } => {
                self.push("var ");
                self.push(name);
                if let Some(t) = ty {
                    self.push(" : ");
                    self.ty(t);
                }
                self.push(" = ");
                self.body(body);
            }
            Stmt::Mutate { name, body } => {
                self.push(name);
                self.push(" <- ");
                self.expr(body);
            }
            Stmt::For {
                binding,
                iter,
                body,
            } => {
                self.push("for ");
                self.pat(binding);
                self.push(" in ");
                self.expr(iter);
                self.push(":");
                self.indented_body(body);
            }
            Stmt::Destructure { pat, body } => {
                self.push("let ");
                self.pat(pat);
                self.push(" = ");
                self.body(body);
            }
            Stmt::TypeDecl(decl) => self.type_decl(decl),
            Stmt::TraitDecl(decl) => self.trait_decl(decl),
            Stmt::Impl(block) => self.impl_block(block),
            Stmt::Tests(cases) => self.tests_block(cases),
            Stmt::Extern { abi, signatures } => {
                self.push("extern \"");
                self.push(abi);
                self.push("\" =");
                self.with_indent(|p| {
                    for sig in signatures {
                        p.newline();
                        p.method_sig(sig);
                    }
                });
            }
            Stmt::Import { path, kind, public } => {
                if *public {
                    self.push("pub ");
                }
                self.push("import ");
                self.push(&path.join("."));
                match kind {
                    ImportKind::All => {}
                    ImportKind::Items(items) => {
                        self.push(".{ ");
                        self.push(&items.join(", "));
                        self.push(" }");
                    }
                    ImportKind::Alias(a) => {
                        self.push(" as ");
                        self.push(a);
                    }
                }
            }
            Stmt::Input { name, body } => {
                self.push("input ");
                self.push(name);
                self.push(" = ");
                self.expr(body);
            }
            Stmt::Output { name, body } => {
                self.push("output ");
                self.push(name);
                self.push(" = ");
                self.expr(body);
            }
            Stmt::Expr(e) => self.expr(e),
        }
    }

    fn let_binding_tail(&mut self, b: &LetBinding) {
        self.push(&b.name);
        for p in &b.params {
            self.push(" ");
            self.param(p);
        }
        if let Some(rt) = &b.return_ty {
            self.push(" : ");
            self.ty(rt);
        }
        self.push(" = ");
        self.body(&b.body);
    }

    fn type_decl(&mut self, decl: &TypeDecl) {
        self.push("type ");
        self.push(&decl.name);
        for p in &decl.params {
            self.push(" '");
            self.push(p);
        }
        self.push(" =");
        match &decl.body {
            TypeDeclBody::Sum(variants) => {
                self.with_indent(|p| {
                    for v in variants {
                        p.newline();
                        p.variant(v);
                    }
                });
            }
            TypeDeclBody::Alias(t) => {
                self.push(" ");
                self.ty(t);
            }
        }
    }

    fn variant(&mut self, v: &TypeVariant) {
        self.push("| ");
        self.push(&v.name);
        for a in &v.args {
            self.push(" ");
            self.ty_atom(a);
        }
    }

    fn trait_decl(&mut self, decl: &TraitDecl) {
        self.push("trait ");
        self.push(&decl.name);
        self.push(" ");
        self.push(&decl.type_var);
        self.push(" =");
        self.with_indent(|p| {
            for m in &decl.methods {
                p.newline();
                p.method_sig(m);
            }
        });
    }

    fn method_sig(&mut self, m: &TraitMethodSig) {
        self.push("fn ");
        self.push(&m.name);
        for p in &m.params {
            self.push(" ");
            self.param(p);
        }
        self.push(" : ");
        self.ty(&m.return_ty);
    }

    fn impl_block(&mut self, b: &ImplBlock) {
        self.push("impl ");
        self.push(&b.trait_name);
        self.push(" ");
        self.ty_atom(&b.ty);
        self.push(" =");
        self.with_indent(|p| {
            for m in &b.methods {
                p.newline();
                p.impl_method(m);
            }
        });
    }

    fn impl_method(&mut self, m: &ImplMethod) {
        self.push("fn ");
        self.push(&m.name);
        for p in &m.params {
            self.push(" ");
            self.param(p);
        }
        if let Some(rt) = &m.return_ty {
            self.push(" : ");
            self.ty(rt);
        }
        self.push(" = ");
        self.body(&m.body);
    }

    fn tests_block(&mut self, cases: &[TestCase]) {
        self.push("tests =");
        self.with_indent(|p| {
            for c in cases {
                p.newline();
                match c {
                    TestCase::Test { name, body } => {
                        p.push("test \"");
                        p.push(name);
                        p.push("\" = ");
                        p.body(body);
                    }
                    TestCase::Prop {
                        name,
                        params,
                        guard,
                        body,
                    } => {
                        p.push("prop \"");
                        p.push(name);
                        p.push("\"");
                        for prm in params {
                            p.push(" ");
                            p.param(prm);
                        }
                        if let Some(g) = guard {
                            p.push(" when ");
                            p.expr(g);
                        }
                        p.push(" = ");
                        p.body(body);
                    }
                }
            }
        });
    }

    fn param(&mut self, p: &Param) {
        match (&p.pat, &p.ty) {
            (Pat::Var(n), None) => self.push(n),
            (Pat::Var(n), Some(t)) => {
                self.push("(");
                self.push(n);
                self.push(" : ");
                self.ty(t);
                self.push(")");
            }
            (pat, None) => self.pat(pat),
            (pat, Some(t)) => {
                self.push("(");
                self.pat(pat);
                self.push(" : ");
                self.ty(t);
                self.push(")");
            }
        }
    }

    fn body(&mut self, e: &Expr) {
        if let Expr::Block { stmts, trailing } = e {
            self.strip_trailing_space();
            self.with_indent(|p| {
                for s in stmts {
                    p.newline();
                    p.stmt(s);
                }
                if let Some(t) = trailing {
                    p.newline();
                    p.expr(t);
                }
            });
        } else {
            self.expr(e);
        }
    }

    fn strip_trailing_space(&mut self) {
        while self.out.ends_with(' ') {
            self.out.pop();
        }
    }

    fn indented_body(&mut self, e: &Expr) {
        self.with_indent(|p| {
            if let Expr::Block { stmts, trailing } = e {
                for s in stmts {
                    p.newline();
                    p.stmt(s);
                }
                if let Some(t) = trailing {
                    p.newline();
                    p.expr(t);
                }
            } else {
                p.newline();
                p.expr(e);
            }
        });
    }

    fn expr(&mut self, e: &Expr) {
        self.expr_bp(e, 0);
    }

    fn expr_bp(&mut self, e: &Expr, parent_bp: u8) {
        let bp = expr_bp(e);
        let needs_parens = bp < parent_bp;
        if needs_parens {
            self.push("(");
        }
        match e {
            Expr::Lit(l) => self.lit(l),
            Expr::Var(name) => self.push(name),
            Expr::Sym(s) => {
                self.push(":");
                self.push(s);
            }
            Expr::UnaryOp(op, inner) => {
                self.push(unop_str(*op));
                if matches!(op, UnOp::Not) {
                    self.push(" ");
                }
                self.expr_bp(inner, 27);
            }
            Expr::BinOp(op, l, r) => {
                let (lbp, rbp) = binop_bp(*op);
                self.expr_bp(l, lbp);
                self.push(" ");
                self.push(binop_str(*op));
                self.push(" ");
                self.expr_bp(r, rbp + 1);
            }
            Expr::Postfix(PostOp::Question, inner) => {
                self.expr_bp(inner, 22);
                self.push("?");
            }
            Expr::App(f, x) => {
                self.expr_bp(f, 25);
                self.push(" ");
                if needs_parens_as_arg(x) {
                    self.push("(");
                    self.expr(x);
                    self.push(")");
                } else {
                    self.expr_bp(x, 26);
                }
            }
            Expr::Lambda(params, body) => {
                self.push("fn");
                for p in params {
                    self.push(" ");
                    self.param(p);
                }
                self.push(" -> ");
                self.body(body);
            }
            Expr::If(c, t, el) => {
                self.push("if ");
                self.expr(c);
                self.push(" then ");
                self.expr(t);
                self.push(" else ");
                self.expr(el);
            }
            Expr::Match(scrut, arms) => {
                self.push("match ");
                self.expr(scrut);
                self.push(" with");
                for arm in arms {
                    self.newline();
                    self.match_arm(arm);
                }
            }
            Expr::Record(fields) => self.record_lit(fields, "{", "}"),
            Expr::RecordUpdate(base, updates) => {
                self.push("{ ");
                self.expr(base);
                self.push(" with ");
                self.field_list(updates);
                self.push(" }");
            }
            Expr::Series(elems) => self.delimited(elems, "[", "]"),
            Expr::Tuple(elems) => self.delimited(elems, "(", ")"),
            Expr::DataFrameLit(cols) => {
                self.push("{|");
                self.with_indent(|p| {
                    for (i, (name, val)) in cols.iter().enumerate() {
                        if i > 0 {
                            p.push(",");
                        }
                        p.newline();
                        p.push(name);
                        p.push(" : ");
                        p.expr(val);
                    }
                    p.push(",");
                });
                self.newline();
                self.push("|}");
            }
            Expr::ArrayLit(rows) => {
                self.push("[| ");
                for (i, row) in rows.iter().enumerate() {
                    if i > 0 {
                        self.push(" ; ");
                    }
                    for (j, x) in row.iter().enumerate() {
                        if j > 0 {
                            self.push(", ");
                        }
                        self.expr(x);
                    }
                }
                self.push(" |]");
            }
            Expr::Field(target, name) => {
                self.expr_bp(target, 28);
                self.push(".");
                self.push(name);
            }
            Expr::Block { stmts, trailing } => {
                self.push("(");
                for (i, s) in stmts.iter().enumerate() {
                    if i > 0 {
                        self.push("; ");
                    }
                    self.stmt(s);
                }
                if let Some(t) = trailing {
                    if !stmts.is_empty() {
                        self.push("; ");
                    }
                    self.expr(t);
                }
                self.push(")");
            }
            Expr::Scope(body) => {
                self.push("scope =");
                self.indented_body(body);
            }
            Expr::Spawn(inner) => {
                self.push("spawn ");
                self.expr_bp(inner, 26);
            }
            Expr::AppBlock(body) => {
                self.push("app =");
                self.indented_body(body);
            }
        }
        if needs_parens {
            self.push(")");
        }
    }

    fn match_arm(&mut self, a: &MatchArm) {
        self.push("| ");
        self.pat(&a.pat);
        if let Some(g) = &a.guard {
            self.push(" when ");
            self.expr(g);
        }
        self.push(" -> ");
        self.body(&a.body);
    }

    fn record_lit(&mut self, fields: &[(String, Expr)], open: &str, close: &str) {
        if fields.is_empty() {
            self.push(open);
            self.push(close);
            return;
        }
        let single = render_inline_fields(fields);
        if single.len() <= 60 {
            self.push(open);
            self.push(" ");
            self.push(&single);
            self.push(" ");
            self.push(close);
        } else {
            self.push(open);
            self.with_indent(|p| {
                for (n, v) in fields {
                    p.newline();
                    p.push(n);
                    p.push(" = ");
                    p.expr(v);
                    p.push(",");
                }
            });
            self.newline();
            self.push(close);
        }
    }

    fn field_list(&mut self, fields: &[(String, Expr)]) {
        for (i, (n, v)) in fields.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            self.push(n);
            self.push(" = ");
            self.expr(v);
        }
    }

    fn delimited(&mut self, elems: &[Expr], open: &str, close: &str) {
        self.push(open);
        for (i, e) in elems.iter().enumerate() {
            if i > 0 {
                self.push(", ");
            }
            self.expr(e);
        }
        self.push(close);
    }

    fn pat(&mut self, p: &Pat) {
        match p {
            Pat::Wildcard => self.push("_"),
            Pat::Var(n) => self.push(n),
            Pat::Lit(l) => self.lit(l),
            Pat::Cons(name, args) => {
                self.push(name);
                for a in args {
                    self.push(" ");
                    self.pat_atom(a);
                }
            }
            Pat::Or(alts) => {
                for (i, a) in alts.iter().enumerate() {
                    if i > 0 {
                        self.push(" | ");
                    }
                    self.pat(a);
                }
            }
            Pat::As(inner, n) => {
                self.pat(inner);
                self.push(" as ");
                self.push(n);
            }
            Pat::List(parts) => {
                self.push("[");
                for (i, p) in parts.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    match p {
                        ListPart::Pat(p) => self.pat(p),
                        ListPart::Rest(Some(n)) => {
                            self.push("..");
                            self.push(n);
                        }
                        ListPart::Rest(None) => self.push(".._"),
                    }
                }
                self.push("]");
            }
            Pat::Range(lo, hi) => {
                self.pat(lo);
                self.push("..=");
                self.pat(hi);
            }
            Pat::Tuple(ps) => {
                self.push("(");
                for (i, p) in ps.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.pat(p);
                }
                self.push(")");
            }
            Pat::Record(fs) => {
                self.push("{ ");
                for (i, (n, p)) in fs.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    if matches!(p, Pat::Var(v) if v == n) {
                        self.push(n);
                    } else {
                        self.push(n);
                        self.push(" = ");
                        self.pat(p);
                    }
                }
                self.push(" }");
            }
        }
    }

    fn pat_atom(&mut self, p: &Pat) {
        match p {
            Pat::Cons(_, args) if !args.is_empty() => {
                self.push("(");
                self.pat(p);
                self.push(")");
            }
            _ => self.pat(p),
        }
    }

    fn ty(&mut self, t: &Ty) {
        match t {
            Ty::Unit => self.push("()"),
            Ty::Con(n) => self.push(n),
            Ty::Var(n) => {
                self.push("'");
                self.push(n);
            }
            Ty::App(base, args) => {
                self.ty(base);
                self.push("[");
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.ty(a);
                }
                self.push("]");
            }
            Ty::Record(fields) => {
                self.push("{ ");
                for (i, (n, t)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.push(n);
                    self.push(" : ");
                    self.ty(t);
                }
                self.push(" }");
            }
            Ty::Series(inner) => {
                self.push("[");
                self.ty(inner);
                self.push("]");
            }
            Ty::Tuple(ts) => {
                self.push("(");
                for (i, t) in ts.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.ty(t);
                }
                self.push(")");
            }
        }
    }

    fn ty_atom(&mut self, t: &Ty) {
        match t {
            Ty::App(_, _) => {
                self.push("(");
                self.ty(t);
                self.push(")");
            }
            _ => self.ty(t),
        }
    }

    fn lit(&mut self, l: &Lit) {
        match l {
            Lit::Int(n) => self.push(&n.to_string()),
            Lit::UInt(n) => self.push(&format!("{n}u")),
            Lit::BigInt(s) => self.push(&format!("{s}n")),
            Lit::Float(f) => {
                if f.is_nan() {
                    self.push("NaN");
                } else if f.is_infinite() {
                    self.push(if *f > 0.0 { "Inf" } else { "-Inf" });
                } else if f.fract() == 0.0 {
                    self.push(&format!("{f:.1}"));
                } else {
                    self.push(&f.to_string());
                }
            }
            Lit::Decimal(s) => self.push(&format!("{s}d")),
            Lit::Str(s) => {
                self.push("\"");
                for c in s.chars() {
                    match c {
                        '\\' => self.push("\\\\"),
                        '"' => self.push("\\\""),
                        '\n' => self.push("\\n"),
                        '\t' => self.push("\\t"),
                        '\r' => self.push("\\r"),
                        _ => self.out.push(c),
                    }
                }
                self.push("\"");
            }
            Lit::Bool(b) => self.push(if *b { "true" } else { "false" }),
            Lit::Unit => self.push("()"),
        }
    }
}

fn import_group(s: &Stmt) -> u8 {
    if let Stmt::Import { path, .. } = s {
        if path.first().is_some_and(|p| p == "std") {
            0
        } else if path.first().is_some_and(|p| p.starts_with('.')) {
            2
        } else {
            1
        }
    } else {
        u8::MAX
    }
}

fn import_sort_key(s: &Stmt) -> (u8, Vec<String>) {
    if let Stmt::Import { path, .. } = s {
        (import_group(s), path.clone())
    } else {
        (u8::MAX, Vec::new())
    }
}

fn render_inline_fields(fields: &[(String, Expr)]) -> String {
    let mut p = Printer::new();
    p.field_list(fields);
    p.out
}

fn unop_str(op: UnOp) -> &'static str {
    match op {
        UnOp::Neg => "-",
        UnOp::Not => "not",
    }
}

fn binop_str(op: BinOp) -> &'static str {
    match op {
        BinOp::Pipe => "|>",
        BinOp::Tilde => "~",
        BinOp::Or => "or",
        BinOp::And => "and",
        BinOp::Eq => "==",
        BinOp::NotEq => "!=",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::Concat => "++",
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Mod => "%",
        BinOp::Pow => "^",
    }
}

fn binop_bp(op: BinOp) -> (u8, u8) {
    match op {
        BinOp::Pipe => (1, 2),
        BinOp::Tilde => (3, 4),
        BinOp::Or => (5, 6),
        BinOp::And => (7, 8),
        BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => (9, 10),
        BinOp::Concat => (11, 12),
        BinOp::Add | BinOp::Sub => (13, 14),
        BinOp::Mul | BinOp::Div | BinOp::Mod => (15, 16),
        BinOp::Pow => (18, 17),
    }
}

fn needs_parens_as_arg(e: &Expr) -> bool {
    matches!(
        e,
        Expr::Lambda(..)
            | Expr::If(..)
            | Expr::Match(..)
            | Expr::Scope(..)
            | Expr::Spawn(..)
            | Expr::AppBlock(..)
    )
}

fn expr_bp(e: &Expr) -> u8 {
    match e {
        Expr::BinOp(op, _, _) => binop_bp(*op).0,
        Expr::UnaryOp(_, _) => 19,
        Expr::Postfix(_, _) => 21,
        Expr::App(_, _) => 25,
        Expr::Field(_, _) => 28,
        _ => 30,
    }
}
