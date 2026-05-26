use vela_parser::{BinOp, Expr, Lit, Stmt, parse_stmt};

fn s(src: &str) -> Stmt {
    parse_stmt(src).expect("parses")
}

fn var(x: &str) -> Expr {
    Expr::Var(x.into())
}
fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}

#[test]
fn simple_let_binding() {
    assert_eq!(
        s("let x = 1"),
        Stmt::Let { name: "x".into(), params: vec![], return_ty: None, body: lit(1), recursive: false },
    );
}

#[test]
fn let_binding_to_expression() {
    assert_eq!(
        s("let total = 1 + 2"),
        Stmt::Let {
            name: "total".into(),
            params: vec![],
            return_ty: None,
            body: Expr::BinOp(BinOp::Add, Box::new(lit(1)), Box::new(lit(2))),
            recursive: false,
        },
    );
}

#[test]
fn let_function_one_param() {
    assert_eq!(
        s("let id x = x"),
        Stmt::Let {
            name: "id".into(),
            params: vec!["x".into()],
            return_ty: None,
            body: var("x"),
            recursive: false,
        },
    );
}

#[test]
fn let_function_two_params() {
    assert_eq!(
        s("let add x y = x + y"),
        Stmt::Let {
            name: "add".into(),
            params: vec!["x".into(), "y".into()],
            return_ty: None,
            body: Expr::BinOp(BinOp::Add, Box::new(var("x")), Box::new(var("y"))),
            recursive: false,
        },
    );
}

#[test]
fn bare_expression_is_a_statement() {
    assert_eq!(s("42"), Stmt::Expr(lit(42)));
}

#[test]
fn var_binding() {
    assert_eq!(
        s("var counter = 0"),
        Stmt::Var { name: "counter".into(), ty: None, body: lit(0) },
    );
}
