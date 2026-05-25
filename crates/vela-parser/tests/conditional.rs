use vela_parser::{BinOp, Expr, Lit, UnOp, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}
fn var(s: &str) -> Expr {
    Expr::Var(s.into())
}
fn if_(c: Expr, t: Expr, e: Expr) -> Expr {
    Expr::If(Box::new(c), Box::new(t), Box::new(e))
}

#[test]
fn simple_if_then_else() {
    assert_eq!(p("if x then 1 else 0"), if_(var("x"), lit(1), lit(0)));
}

#[test]
fn if_with_comparison() {
    assert_eq!(
        p("if x > 0 then x else -x"),
        if_(
            Expr::BinOp(BinOp::Gt, Box::new(var("x")), Box::new(lit(0))),
            var("x"),
            Expr::UnaryOp(UnOp::Neg, Box::new(var("x"))),
        ),
    );
}

#[test]
fn nested_if() {
    assert_eq!(
        p("if a then if b then 1 else 2 else 3"),
        if_(var("a"), if_(var("b"), lit(1), lit(2)), lit(3)),
    );
}

#[test]
fn branches_extend_greedily() {
    assert_eq!(
        p("if c then 1 + 2 else 3 + 4"),
        if_(
            var("c"),
            Expr::BinOp(BinOp::Add, Box::new(lit(1)), Box::new(lit(2))),
            Expr::BinOp(BinOp::Add, Box::new(lit(3)), Box::new(lit(4))),
        ),
    );
}

#[test]
fn if_inside_arithmetic() {
    // 1 + if a then 2 else 3
    // → 1 + (if a then 2 else 3)
    assert_eq!(
        p("1 + if a then 2 else 3"),
        Expr::BinOp(
            BinOp::Add,
            Box::new(lit(1)),
            Box::new(if_(var("a"), lit(2), lit(3))),
        ),
    );
}
