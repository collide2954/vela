use vela_parser::{Expr, Lit, parse_expr};

fn parse(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

#[test]
fn int_literal() {
    assert_eq!(parse("42"), Expr::Lit(Lit::Int(42)));
}

#[test]
fn float_literal() {
    assert_eq!(parse("3.5"), Expr::Lit(Lit::Float(3.5)));
}

#[test]
fn string_literal() {
    assert_eq!(parse(r#""hello""#), Expr::Lit(Lit::Str("hello".into())));
}

#[test]
fn true_literal() {
    assert_eq!(parse("true"), Expr::Lit(Lit::Bool(true)));
}

#[test]
fn false_literal() {
    assert_eq!(parse("false"), Expr::Lit(Lit::Bool(false)));
}

#[test]
fn unit_literal() {
    assert_eq!(parse("()"), Expr::Lit(Lit::Unit));
}
