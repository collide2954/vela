use vela_parser::{BinOp, Expr, Lit, Stmt, parse_stmt};

fn s(src: &str) -> Stmt {
    parse_stmt(src).expect("parses")
}

fn var(s: &str) -> Expr {
    Expr::Var(s.into())
}
fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}

#[test]
fn simple_mutation() {
    assert_eq!(
        s("x <- y"),
        Stmt::Mutate {
            name: "x".into(),
            body: var("y")
        },
    );
}

#[test]
fn mutation_with_expression() {
    assert_eq!(
        s("counter <- counter + 1"),
        Stmt::Mutate {
            name: "counter".into(),
            body: Expr::BinOp(BinOp::Add, Box::new(var("counter")), Box::new(lit(1))),
        },
    );
}

#[test]
fn mutation_to_literal() {
    assert_eq!(
        s("x <- 0"),
        Stmt::Mutate {
            name: "x".into(),
            body: lit(0)
        },
    );
}
