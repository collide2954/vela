use vela_parser::{Expr, Lit, Pat, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}
fn var(s: &str) -> Expr {
    Expr::Var(s.into())
}
fn cons(name: &str, args: Vec<Pat>) -> Pat {
    Pat::Cons(name.into(), args)
}

#[test]
fn or_pattern_two_alternatives() {
    let e = p(r#"match x with | Red | Blue -> "primary""#);
    if let Expr::Match(_, arms) = e {
        assert_eq!(arms.len(), 1);
        assert_eq!(arms[0].pat, Pat::Or(vec![cons("Red", vec![]), cons("Blue", vec![])]));
    } else {
        panic!("expected match");
    }
}

#[test]
fn or_pattern_three_alternatives() {
    let e = p("match x with | A | B | C -> 1");
    if let Expr::Match(_, arms) = e {
        assert_eq!(
            arms[0].pat,
            Pat::Or(vec![cons("A", vec![]), cons("B", vec![]), cons("C", vec![])]),
        );
    } else {
        panic!("expected match");
    }
}

#[test]
fn or_pattern_does_not_eat_next_arm() {
    let e = p("match x with | A | B -> 1 | C -> 2");
    if let Expr::Match(_, arms) = e {
        assert_eq!(arms.len(), 2);
        assert_eq!(arms[0].pat, Pat::Or(vec![cons("A", vec![]), cons("B", vec![])]));
        assert_eq!(arms[0].body, lit(1));
        assert_eq!(arms[1].pat, cons("C", vec![]));
        assert_eq!(arms[1].body, lit(2));
    } else {
        panic!("expected match");
    }
}

#[test]
fn as_binding_on_constructor() {
    let e = p("match s with | Circle r as c -> c");
    if let Expr::Match(_, arms) = e {
        assert_eq!(
            arms[0].pat,
            Pat::As(
                Box::new(cons("Circle", vec![Pat::Var("r".into())])),
                "c".into(),
            ),
        );
        assert_eq!(arms[0].body, var("c"));
    } else {
        panic!("expected match");
    }
}

#[test]
fn as_binding_on_wildcard() {
    let e = p("match v with | _ as everything -> everything");
    if let Expr::Match(_, arms) = e {
        assert_eq!(
            arms[0].pat,
            Pat::As(Box::new(Pat::Wildcard), "everything".into()),
        );
    } else {
        panic!("expected match");
    }
}

#[test]
fn or_pattern_with_payload_constructors() {
    let e = p("match s with | Square s | Circle s -> s");
    if let Expr::Match(_, arms) = e {
        assert_eq!(
            arms[0].pat,
            Pat::Or(vec![
                cons("Square", vec![Pat::Var("s".into())]),
                cons("Circle", vec![Pat::Var("s".into())]),
            ]),
        );
    } else {
        panic!("expected match");
    }
}

#[test]
fn singleton_arm_is_not_or_pattern() {
    let e = p(r#"match x with | Red -> "red""#);
    if let Expr::Match(_, arms) = e {
        assert_eq!(arms[0].pat, cons("Red", vec![]));
        assert!(!matches!(arms[0].pat, Pat::Or(_)));
    } else {
        panic!("expected match");
    }
}
