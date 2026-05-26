use vela_parser::{Param, Pat, Stmt, parse_stmt};

fn s(src: &str) -> Stmt {
    parse_stmt(src).expect("parses")
}

#[test]
fn unit_param() {
    if let Stmt::Let { params, .. } = s("let thunk () = 42") {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].pat, Pat::Lit(vela_parser::Lit::Unit));
    } else {
        panic!("expected let");
    }
}

#[test]
fn tuple_pattern_param() {
    if let Stmt::Let { params, .. } = s("let dist (a, b) = a + b") {
        assert_eq!(params.len(), 1);
        assert_eq!(
            params[0].pat,
            Pat::Tuple(vec![Pat::Var("a".into()), Pat::Var("b".into())]),
        );
    } else {
        panic!("expected let");
    }
}

#[test]
fn record_pattern_param() {
    if let Stmt::Let { params, .. } = s("let area { width = w, height = h } = w * h") {
        assert_eq!(params.len(), 1);
        if let Pat::Record(fields) = &params[0].pat {
            assert_eq!(fields.len(), 2);
        } else {
            panic!("expected record pattern");
        }
    } else {
        panic!("expected let");
    }
}

#[test]
fn mixed_simple_and_pattern_params() {
    if let Stmt::Let { params, .. } = s("let f x () (a, b) = x") {
        assert_eq!(params.len(), 3);
        assert!(matches!(params[0].pat, Pat::Var(_)));
        assert!(matches!(params[1].pat, Pat::Lit(_)));
        assert!(matches!(params[2].pat, Pat::Tuple(_)));
    } else {
        panic!("expected let");
    }
}

#[test]
fn simple_name_helper() {
    let p: Param = "x".into();
    assert_eq!(p.simple_name(), Some("x"));
    let p2 = Param {
        pat: Pat::Lit(vela_parser::Lit::Unit),
        ty: None,
    };
    assert_eq!(p2.simple_name(), None);
}
