use vela_parser::{Expr, Lit, Pat, Stmt, parse_stmt};

fn s(src: &str) -> Stmt {
    parse_stmt(src).expect("parses")
}

#[test]
fn tuple_destructuring() {
    assert_eq!(
        s(r#"let (a, b) = (1, "x")"#),
        Stmt::Destructure {
            pat: Pat::Tuple(vec![Pat::Var("a".into()), Pat::Var("b".into())]),
            body: Expr::Tuple(vec![
                Expr::Lit(Lit::Int(1)),
                Expr::Lit(Lit::Str("x".into())),
            ]),
        },
    );
}

#[test]
fn tuple_with_wildcard() {
    assert_eq!(
        s("let (a, _) = (1, 2)"),
        Stmt::Destructure {
            pat: Pat::Tuple(vec![Pat::Var("a".into()), Pat::Wildcard]),
            body: Expr::Tuple(vec![Expr::Lit(Lit::Int(1)), Expr::Lit(Lit::Int(2))]),
        },
    );
}

#[test]
fn record_destructuring_punning() {
    assert_eq!(
        s("let { x, y } = pt"),
        Stmt::Destructure {
            pat: Pat::Record(vec![
                ("x".into(), Pat::Var("x".into())),
                ("y".into(), Pat::Var("y".into())),
            ]),
            body: Expr::Var("pt".into()),
        },
    );
}

#[test]
fn record_destructuring_explicit() {
    assert_eq!(
        s("let { x = a, y = b } = pt"),
        Stmt::Destructure {
            pat: Pat::Record(vec![
                ("x".into(), Pat::Var("a".into())),
                ("y".into(), Pat::Var("b".into())),
            ]),
            body: Expr::Var("pt".into()),
        },
    );
}

#[test]
fn list_destructuring_head_tail() {
    assert_eq!(
        s("let [head, ..tail] = xs"),
        Stmt::Destructure {
            pat: Pat::List(vec![
                vela_parser::ListPart::Pat(Pat::Var("head".into())),
                vela_parser::ListPart::Rest(Some("tail".into())),
            ]),
            body: Expr::Var("xs".into()),
        },
    );
}
