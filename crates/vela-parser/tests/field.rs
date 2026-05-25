use vela_parser::{BinOp, Expr, Lit, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn var(s: &str) -> Expr {
    Expr::Var(s.into())
}
fn field(e: Expr, n: &str) -> Expr {
    Expr::Field(Box::new(e), n.into())
}
fn app(f: Expr, x: Expr) -> Expr {
    Expr::App(Box::new(f), Box::new(x))
}

#[test]
fn simple_field_access() {
    assert_eq!(p("df.x"), field(var("df"), "x"));
}

#[test]
fn chained_field_access() {
    assert_eq!(p("df.x.y"), field(field(var("df"), "x"), "y"));
}

#[test]
fn field_access_in_arithmetic() {
    assert_eq!(
        p("point.x + point.y"),
        Expr::BinOp(
            BinOp::Add,
            Box::new(field(var("point"), "x")),
            Box::new(field(var("point"), "y")),
        ),
    );
}

#[test]
fn field_binds_tighter_than_application() {
    // f x.y → f (x.y)
    assert_eq!(p("f x.y"), app(var("f"), field(var("x"), "y")));
}

#[test]
fn field_then_application() {
    // f.x y → (f.x) y
    assert_eq!(p("f.x y"), app(field(var("f"), "x"), var("y")));
}

#[test]
fn field_on_record_literal() {
    assert_eq!(
        p("{ x = 1 }.x"),
        field(Expr::Record(vec![("x".into(), Expr::Lit(Lit::Int(1)))]), "x"),
    );
}
