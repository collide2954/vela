use vela_parser::{Expr, Lit, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn flit(f: f64) -> Expr {
    Expr::Lit(Lit::Float(f))
}
fn ilit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}

#[test]
fn empty_array() {
    assert_eq!(p("[||]"), Expr::ArrayLit(vec![]));
}

#[test]
fn one_d_array() {
    assert_eq!(
        p("[| 1, 2, 3 |]"),
        Expr::ArrayLit(vec![vec![ilit(1), ilit(2), ilit(3)]]),
    );
}

#[test]
fn two_d_array() {
    assert_eq!(
        p("[| 1, 2 ; 3, 4 |]"),
        Expr::ArrayLit(vec![vec![ilit(1), ilit(2)], vec![ilit(3), ilit(4)]]),
    );
}

#[test]
fn three_row_array() {
    assert_eq!(
        p("[| 1, 2 ; 3, 4 ; 5, 6 |]"),
        Expr::ArrayLit(vec![
            vec![ilit(1), ilit(2)],
            vec![ilit(3), ilit(4)],
            vec![ilit(5), ilit(6)],
        ]),
    );
}

#[test]
fn float_array() {
    assert_eq!(
        p("[| 1.0, 2.0 ; 3.0, 4.0 |]"),
        Expr::ArrayLit(vec![
            vec![flit(1.0), flit(2.0)],
            vec![flit(3.0), flit(4.0)],
        ]),
    );
}

#[test]
fn single_element_array() {
    assert_eq!(p("[| 42 |]"), Expr::ArrayLit(vec![vec![ilit(42)]]));
}
