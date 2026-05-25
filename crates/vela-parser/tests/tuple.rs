use vela_parser::{Expr, Lit, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}
fn slit(s: &str) -> Expr {
    Expr::Lit(Lit::Str(s.into()))
}

#[test]
fn two_element_tuple() {
    assert_eq!(p("(1, 2)"), Expr::Tuple(vec![lit(1), lit(2)]));
}

#[test]
fn three_element_tuple() {
    assert_eq!(p("(1, 2, 3)"), Expr::Tuple(vec![lit(1), lit(2), lit(3)]));
}

#[test]
fn mixed_type_tuple() {
    assert_eq!(p(r#"(1, "a")"#), Expr::Tuple(vec![lit(1), slit("a")]));
}

#[test]
fn tuple_with_trailing_comma() {
    assert_eq!(p("(1, 2,)"), Expr::Tuple(vec![lit(1), lit(2)]));
}

#[test]
fn parenthesized_single_value_is_not_a_tuple() {
    assert_eq!(p("(42)"), lit(42));
}

#[test]
fn unit_remains_unit() {
    assert_eq!(p("()"), Expr::Lit(Lit::Unit));
}

#[test]
fn nested_tuple() {
    assert_eq!(
        p("((1, 2), 3)"),
        Expr::Tuple(vec![Expr::Tuple(vec![lit(1), lit(2)]), lit(3)]),
    );
}
