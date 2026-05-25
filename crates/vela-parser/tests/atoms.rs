use vela_parser::{Expr, Lit, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

#[test]
fn simple_identifier() {
    assert_eq!(p("foo"), Expr::Var("foo".into()));
}

#[test]
fn snake_case_identifier() {
    assert_eq!(p("snake_case_42"), Expr::Var("snake_case_42".into()));
}

#[test]
fn upper_camel_identifier() {
    assert_eq!(p("DataFrame"), Expr::Var("DataFrame".into()));
}

#[test]
fn parens_around_literal_unwrap() {
    assert_eq!(p("(42)"), Expr::Lit(Lit::Int(42)));
}

#[test]
fn parens_around_identifier_unwrap() {
    assert_eq!(p("(foo)"), Expr::Var("foo".into()));
}

#[test]
fn nested_parens_unwrap() {
    assert_eq!(p("(((7)))"), Expr::Lit(Lit::Int(7)));
}
