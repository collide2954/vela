use vela_parser::{Expr, Lit, Stmt, parse_program};

fn p(src: &str) -> Vec<Stmt> {
    parse_program(src).expect("parses").stmts
}

fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}

#[test]
fn empty_program() {
    assert_eq!(p(""), vec![]);
}

#[test]
fn single_let_statement() {
    assert_eq!(
        p("let x = 1\n"),
        vec![Stmt::Let { name: "x".into(), params: vec![], return_ty: None, body: lit(1), recursive: false }],
    );
}

#[test]
fn two_statements_newline_separated() {
    assert_eq!(
        p("let x = 1\nlet y = 2\n"),
        vec![
            Stmt::Let { name: "x".into(), params: vec![], return_ty: None, body: lit(1), recursive: false },
            Stmt::Let { name: "y".into(), params: vec![], return_ty: None, body: lit(2), recursive: false },
        ],
    );
}

#[test]
fn trailing_expression_statement() {
    assert_eq!(
        p("let x = 1\nx"),
        vec![
            Stmt::Let { name: "x".into(), params: vec![], return_ty: None, body: lit(1), recursive: false },
            Stmt::Expr(Expr::Var("x".into())),
        ],
    );
}

#[test]
fn no_trailing_newline_ok() {
    assert_eq!(
        p("let x = 1"),
        vec![Stmt::Let { name: "x".into(), params: vec![], return_ty: None, body: lit(1), recursive: false }],
    );
}

#[test]
fn blank_lines_between_statements() {
    assert_eq!(
        p("let x = 1\n\nlet y = 2\n"),
        vec![
            Stmt::Let { name: "x".into(), params: vec![], return_ty: None, body: lit(1), recursive: false },
            Stmt::Let { name: "y".into(), params: vec![], return_ty: None, body: lit(2), recursive: false },
        ],
    );
}

#[test]
fn comment_only_lines_are_skipped() {
    assert_eq!(
        p("let x = 1\n# nothing\nlet y = 2"),
        vec![
            Stmt::Let { name: "x".into(), params: vec![], return_ty: None, body: lit(1), recursive: false },
            Stmt::Let { name: "y".into(), params: vec![], return_ty: None, body: lit(2), recursive: false },
        ],
    );
}
