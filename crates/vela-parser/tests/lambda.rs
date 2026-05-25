use vela_parser::{BinOp, Expr, Lit, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn var(s: &str) -> Expr {
    Expr::Var(s.into())
}
fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}
fn lam(params: &[&str], body: Expr) -> Expr {
    Expr::Lambda(params.iter().map(|s| s.to_string()).collect(), Box::new(body))
}

#[test]
fn one_param_lambda() {
    assert_eq!(p("fn x -> x"), lam(&["x"], var("x")));
}

#[test]
fn two_param_lambda() {
    assert_eq!(
        p("fn x y -> x + y"),
        lam(
            &["x", "y"],
            Expr::BinOp(BinOp::Add, Box::new(var("x")), Box::new(var("y"))),
        ),
    );
}

#[test]
fn lambda_body_extends_to_pipe() {
    assert_eq!(
        p("fn x -> x + 1"),
        lam(
            &["x"],
            Expr::BinOp(BinOp::Add, Box::new(var("x")), Box::new(lit(1))),
        ),
    );
}

#[test]
fn lambda_as_argument() {
    assert_eq!(
        p("map (fn x -> x + 1) xs"),
        Expr::App(
            Box::new(Expr::App(
                Box::new(var("map")),
                Box::new(lam(
                    &["x"],
                    Expr::BinOp(BinOp::Add, Box::new(var("x")), Box::new(lit(1))),
                )),
            )),
            Box::new(var("xs")),
        ),
    );
}
