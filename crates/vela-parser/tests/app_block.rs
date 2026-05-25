use vela_parser::{Expr, Stmt, parse_program, parse_stmt};

fn s(src: &str) -> Stmt {
    parse_stmt(src).expect("parses")
}

#[test]
fn input_statement() {
    let stmt = s("input n = slider");
    if let Stmt::Input { name, .. } = stmt {
        assert_eq!(name, "n");
    } else {
        panic!("expected input");
    }
}

#[test]
fn output_statement() {
    let stmt = s("output table = df");
    if let Stmt::Output { name, .. } = stmt {
        assert_eq!(name, "table");
    } else {
        panic!("expected output");
    }
}

#[test]
fn app_block_with_input_let_output() {
    let program = parse_program(
        "let dashboard = app =\n    input n = slider\n    let df = generate n\n    output t = df\n",
    )
    .expect("parses");
    assert_eq!(program.stmts.len(), 1);
    if let Stmt::Let { body: Expr::AppBlock(inner), .. } = &program.stmts[0] {
        if let Expr::Block { stmts, trailing } = &**inner {
            assert_eq!(stmts.len(), 3);
            assert!(matches!(stmts[0], Stmt::Input { .. }));
            assert!(matches!(stmts[1], Stmt::Let { .. }));
            assert!(matches!(stmts[2], Stmt::Output { .. }));
            assert!(trailing.is_none());
        } else {
            panic!("expected block inside app");
        }
    } else {
        panic!("expected let with app body, got {:?}", program.stmts[0]);
    }
}
