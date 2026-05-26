use vela_parser::{Expr, Lit, Stmt, parse_expr, parse_program};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

#[test]
fn spawn_a_call() {
    assert_eq!(
        p(r#"spawn (load "a.csv")"#),
        Expr::Spawn(Box::new(Expr::App(
            Box::new(Expr::Var("load".into())),
            Box::new(Expr::Lit(Lit::Str("a.csv".into()))),
        ))),
    );
}

#[test]
fn scope_block_with_two_spawns() {
    let program = parse_program("let result = scope =\n    spawn (load 1)\n    spawn (load 2)\n")
        .expect("parses");
    assert_eq!(program.stmts.len(), 1);
    if let Stmt::Let { body, .. } = &program.stmts[0] {
        assert!(matches!(body, Expr::Scope(_)));
    } else {
        panic!("expected let with scope body");
    }
}

#[test]
fn scope_body_contains_block_with_spawns() {
    let program =
        parse_program("let r = scope =\n    spawn (f 1)\n    spawn (f 2)\n").expect("parses");
    if let Stmt::Let {
        body: Expr::Scope(inner),
        ..
    } = &program.stmts[0]
    {
        if let Expr::Block { stmts, trailing } = &**inner {
            assert_eq!(stmts.len(), 1);
            assert!(trailing.is_some());
            assert!(matches!(stmts[0], Stmt::Expr(Expr::Spawn(_))));
        } else {
            panic!("expected block inside scope");
        }
    } else {
        panic!("expected scope expression");
    }
}
