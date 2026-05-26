use vela_parser::{Expr, Lit, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}
fn flit(f: f64) -> Expr {
    Expr::Lit(Lit::Float(f))
}
fn slit(s: &str) -> Expr {
    Expr::Lit(Lit::Str(s.into()))
}

#[test]
fn empty_series() {
    assert_eq!(p("[]"), Expr::Series(vec![]));
}

#[test]
fn single_element_series() {
    assert_eq!(p("[1]"), Expr::Series(vec![lit(1)]));
}

#[test]
fn integer_series() {
    assert_eq!(p("[1, 2, 3]"), Expr::Series(vec![lit(1), lit(2), lit(3)]));
}

#[test]
fn float_series() {
    assert_eq!(
        p("[1.0, 2.5, 3.0]"),
        Expr::Series(vec![flit(1.0), flit(2.5), flit(3.0)])
    );
}

#[test]
fn string_series() {
    assert_eq!(p(r#"["a", "b"]"#), Expr::Series(vec![slit("a"), slit("b")]));
}

#[test]
fn trailing_comma_allowed() {
    assert_eq!(p("[1, 2,]"), Expr::Series(vec![lit(1), lit(2)]));
}

#[test]
fn nested_series() {
    assert_eq!(
        p("[[1], [2, 3]]"),
        Expr::Series(vec![
            Expr::Series(vec![lit(1)]),
            Expr::Series(vec![lit(2), lit(3)]),
        ]),
    );
}
