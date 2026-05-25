use vela_parser::{Expr, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn var(s: &str) -> Expr {
    Expr::Var(s.into())
}
fn sym(s: &str) -> Expr {
    Expr::Sym(s.into())
}
fn app(f: Expr, x: Expr) -> Expr {
    Expr::App(Box::new(f), Box::new(x))
}

#[test]
fn simple_symbol() {
    assert_eq!(p(":species"), sym("species"));
}

#[test]
fn symbol_with_digits() {
    assert_eq!(p(":petal_length_2"), sym("petal_length_2"));
}

#[test]
fn symbol_as_function_argument() {
    assert_eq!(p("col :species"), app(var("col"), sym("species")));
}

#[test]
fn group_by_symbol() {
    assert_eq!(p("group_by :x"), app(var("group_by"), sym("x")));
}
