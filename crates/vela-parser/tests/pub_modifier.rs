use vela_parser::{Stmt, parse_program, parse_stmt};

#[test]
fn pub_let_parses_as_regular_let() {
    let stmt = parse_stmt("pub let mean xs = xs").expect("parses");
    assert!(matches!(stmt, Stmt::Let { ref name, .. } if name == "mean"));
}

#[test]
fn pub_var() {
    let stmt = parse_stmt("pub var counter = 0").expect("parses");
    assert!(matches!(stmt, Stmt::Var { ref name, .. } if name == "counter"));
}

#[test]
fn pub_type() {
    let stmt = parse_stmt("pub type Color = | Red | Blue").expect("parses");
    assert!(matches!(stmt, Stmt::TypeDecl(_)));
}

#[test]
fn pub_program_mix() {
    let prog =
        parse_program("pub let mean xs = xs\npub type Color = | Red | Blue\nlet private = 1")
            .expect("parses");
    assert_eq!(prog.stmts.len(), 3);
}
