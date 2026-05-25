use vela_parser::{BinOp, Expr, Lit, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}
fn lit_f(f: f64) -> Expr {
    Expr::Lit(Lit::Float(f))
}
fn var(s: &str) -> Expr {
    Expr::Var(s.into())
}
fn rec(fields: &[(&str, Expr)]) -> Expr {
    Expr::Record(fields.iter().map(|(n, e)| ((*n).into(), e.clone())).collect())
}

#[test]
fn empty_record() {
    assert_eq!(p("{}"), Expr::Record(vec![]));
}

#[test]
fn single_field_record() {
    assert_eq!(p("{ x = 1 }"), rec(&[("x", lit(1))]));
}

#[test]
fn two_field_record() {
    assert_eq!(p("{ x = 1, y = 2 }"), rec(&[("x", lit(1)), ("y", lit(2))]));
}

#[test]
fn trailing_comma_allowed() {
    assert_eq!(p("{ x = 1, }"), rec(&[("x", lit(1))]));
}

#[test]
fn expression_as_field_value() {
    assert_eq!(
        p("{ total = 1 + 2 }"),
        rec(&[(
            "total",
            Expr::BinOp(BinOp::Add, Box::new(lit(1)), Box::new(lit(2))),
        )]),
    );
}

#[test]
fn record_update() {
    assert_eq!(
        p("{ point with x = 3.0 }"),
        Expr::RecordUpdate(Box::new(var("point")), vec![("x".into(), lit_f(3.0))]),
    );
}

#[test]
fn record_update_multiple_fields() {
    assert_eq!(
        p("{ p with x = 1, y = 2 }"),
        Expr::RecordUpdate(
            Box::new(var("p")),
            vec![("x".into(), lit(1)), ("y".into(), lit(2))],
        ),
    );
}
