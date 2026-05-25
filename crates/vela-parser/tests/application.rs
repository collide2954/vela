use vela_parser::{BinOp, Expr, Lit, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}
fn var(s: &str) -> Expr {
    Expr::Var(s.into())
}
fn app(f: Expr, x: Expr) -> Expr {
    Expr::App(Box::new(f), Box::new(x))
}
fn bin(op: BinOp, a: Expr, b: Expr) -> Expr {
    Expr::BinOp(op, Box::new(a), Box::new(b))
}

#[test]
fn simple_application() {
    assert_eq!(p("f x"), app(var("f"), var("x")));
}

#[test]
fn curried_application_is_left_associative() {
    assert_eq!(p("f x y"), app(app(var("f"), var("x")), var("y")));
}

#[test]
fn application_binds_tighter_than_addition() {
    assert_eq!(
        p("f x + g y"),
        bin(BinOp::Add, app(var("f"), var("x")), app(var("g"), var("y"))),
    );
}

#[test]
fn parens_group_application_argument() {
    assert_eq!(p("f (g x)"), app(var("f"), app(var("g"), var("x"))));
}

#[test]
fn application_to_literal() {
    assert_eq!(p("f 42"), app(var("f"), lit(42)));
}

#[test]
fn application_through_pipe() {
    assert_eq!(
        p("1 |> f"),
        Expr::BinOp(BinOp::Pipe, Box::new(lit(1)), Box::new(var("f"))),
    );
}

#[test]
fn three_arg_application_with_literal_args() {
    assert_eq!(
        p("add 1 2"),
        app(app(var("add"), lit(1)), lit(2)),
    );
}

#[test]
fn application_then_postfix_question() {
    assert_eq!(
        p("read x?"),
        Expr::Postfix(
            vela_parser::PostOp::Question,
            Box::new(app(var("read"), var("x"))),
        ),
    );
}
