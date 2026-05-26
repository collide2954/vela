use vela_parser::{Expr, Lit, Pat, Stmt, parse_stmt};

fn s(src: &str) -> Stmt {
    parse_stmt(src).expect("parses")
}

fn var(x: &str) -> Expr {
    Expr::Var(x.into())
}
fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}
fn app(f: Expr, x: Expr) -> Expr {
    Expr::App(Box::new(f), Box::new(x))
}

#[test]
fn single_line_for_loop() {
    assert_eq!(
        s("for x in xs: println x"),
        Stmt::For {
            binding: Pat::Var("x".into()),
            iter: var("xs"),
            body: app(var("println"), var("x")),
        },
    );
}

#[test]
fn for_loop_with_indented_block() {
    let stmt = s("for x in xs:\n    println x\n    println x");
    if let Stmt::For { binding, iter, body } = stmt {
        assert_eq!(binding, Pat::Var("x".into()));
        assert_eq!(iter, var("xs"));
        if let Expr::Block { stmts, trailing } = body {
            assert_eq!(stmts.len(), 1);
            assert!(trailing.is_some());
        } else {
            panic!("expected block body");
        }
    } else {
        panic!("expected for loop");
    }
}

#[test]
fn for_loop_with_mutation() {
    let stmt = s("for x in xs:\n    total <- total + x");
    if let Stmt::For { binding, body, .. } = stmt {
        assert_eq!(binding, Pat::Var("x".into()));
        match body {
            Expr::Block { stmts, .. } => {
                assert_eq!(stmts.len(), 1);
                assert!(matches!(stmts[0], Stmt::Mutate { .. }));
            }
            Expr::BinOp(..) => panic!("singleton should not flatten mutate"),
            other => panic!("expected block, got {other:?}"),
        }
    } else {
        panic!("expected for loop");
    }
}

#[test]
fn for_loop_over_range() {
    assert_eq!(
        s("for i in r: process i"),
        Stmt::For {
            binding: Pat::Var("i".into()),
            iter: var("r"),
            body: app(var("process"), var("i")),
        },
    );
}

#[test]
fn for_body_single_expression_unwraps() {
    let stmt = s("for x in xs:\n    println x");
    if let Stmt::For { body, .. } = stmt {
        assert_eq!(body, app(var("println"), var("x")));
    } else {
        panic!("expected for loop");
    }
}

// Sanity: existing let statement still works alongside the new branch
#[test]
fn let_still_works() {
    assert_eq!(
        s("let x = 1"),
        Stmt::Let { name: "x".into(), params: vec![], return_ty: None, body: lit(1), recursive: false },
    );
}
