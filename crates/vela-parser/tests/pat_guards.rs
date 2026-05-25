use vela_parser::{BinOp, Expr, Lit, Pat, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}
fn var_e(s: &str) -> Expr {
    Expr::Var(s.into())
}

#[test]
fn simple_guard() {
    let e = p("match x with | n when n > 0 -> n | _ -> 0");
    if let Expr::Match(_, arms) = e {
        assert_eq!(arms.len(), 2);
        assert_eq!(arms[0].pat, Pat::Var("n".into()));
        assert_eq!(
            arms[0].guard,
            Some(Expr::BinOp(
                BinOp::Gt,
                Box::new(var_e("n")),
                Box::new(lit(0)),
            )),
        );
        assert_eq!(arms[0].body, var_e("n"));
        assert_eq!(arms[1].guard, None);
    } else {
        panic!("expected match");
    }
}

#[test]
fn no_guard_yields_none() {
    let e = p("match x with | 0 -> 1");
    if let Expr::Match(_, arms) = e {
        assert_eq!(arms[0].guard, None);
    } else {
        panic!("expected match");
    }
}

#[test]
fn guard_on_constructor_pattern() {
    let e = p("match shape with | Circle r when r > 0.0 -> 1 | _ -> 0");
    if let Expr::Match(_, arms) = e {
        assert!(arms[0].guard.is_some());
        if let Some(Expr::BinOp(op, ..)) = &arms[0].guard {
            assert_eq!(*op, BinOp::Gt);
        } else {
            panic!("expected binop guard");
        }
    } else {
        panic!("expected match");
    }
}

#[test]
fn guard_with_or_pattern() {
    let e = p("match x with | A | B when valid x -> 1 | _ -> 0");
    if let Expr::Match(_, arms) = e {
        assert_eq!(arms.len(), 2);
        assert!(matches!(arms[0].pat, Pat::Or(_)));
        assert!(arms[0].guard.is_some());
    } else {
        panic!("expected match");
    }
}
