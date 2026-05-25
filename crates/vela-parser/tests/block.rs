use vela_parser::{BinOp, Expr, Lit, Stmt, parse_program, parse_stmt};

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
fn single_line_let_body_unchanged() {
    assert_eq!(
        s("let x = 1"),
        Stmt::Let { name: "x".into(), params: vec![], body: lit(1) },
    );
}

#[test]
fn indented_singleton_block_unwraps() {
    assert_eq!(
        s("let x =\n    1"),
        Stmt::Let { name: "x".into(), params: vec![], body: lit(1) },
    );
}

#[test]
fn indented_multi_statement_block() {
    let stmt = s("let f y =\n    let z = y + 1\n    z * 2");
    let expected_body = Expr::Block {
        stmts: vec![Stmt::Let {
            name: "z".into(),
            params: vec![],
            body: Expr::BinOp(BinOp::Add, Box::new(var("y")), Box::new(lit(1))),
        }],
        trailing: Some(Box::new(Expr::BinOp(
            BinOp::Mul,
            Box::new(var("z")),
            Box::new(lit(2)),
        ))),
    };
    assert_eq!(
        stmt,
        Stmt::Let { name: "f".into(), params: vec!["y".into()], body: expected_body },
    );
}

#[test]
fn nested_let_in_block() {
    let stmt = s("let outer =\n    let inner = 1\n    inner + 2");
    if let Stmt::Let { body: Expr::Block { stmts, trailing }, .. } = stmt {
        assert_eq!(stmts.len(), 1);
        assert!(trailing.is_some());
    } else {
        panic!("expected let with block body");
    }
}

#[test]
fn program_with_indented_function() {
    let program = parse_program(
        "let id x =\n    x\n\nlet add x y =\n    x + y\n",
    )
    .expect("parses");
    assert_eq!(program.stmts.len(), 2);
}
