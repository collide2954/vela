use vela_parser::{Expr, Lit, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}
fn flit(f: f64) -> Expr {
    Expr::Lit(Lit::Float(f))
}
fn slit(s: &str) -> Expr {
    Expr::Lit(Lit::Str(s.into()))
}

#[test]
fn empty_dataframe() {
    assert_eq!(p("{||}"), Expr::DataFrameLit(vec![]));
}

#[test]
fn single_column_dataframe() {
    assert_eq!(
        p("{| x : [1, 2, 3] |}"),
        Expr::DataFrameLit(vec![("x".into(), Expr::Series(vec![lit(1), lit(2), lit(3)]))]),
    );
}

#[test]
fn multi_column_dataframe() {
    assert_eq!(
        p(r#"{| name : ["a", "b"], x : [1.0, 2.0] |}"#),
        Expr::DataFrameLit(vec![
            ("name".into(), Expr::Series(vec![slit("a"), slit("b")])),
            ("x".into(), Expr::Series(vec![flit(1.0), flit(2.0)])),
        ]),
    );
}

#[test]
fn trailing_comma_allowed() {
    assert_eq!(
        p("{| x : [1, 2], |}"),
        Expr::DataFrameLit(vec![("x".into(), Expr::Series(vec![lit(1), lit(2)]))]),
    );
}
