use vela_parser::{Expr, Lit, Pat, parse_expr};

fn arm_pat(src: &str) -> Pat {
    if let Expr::Match(_, arms) = parse_expr(src).expect("parses") {
        arms[0].pat.clone()
    } else {
        panic!("expected match");
    }
}

fn lit_pat(n: i64) -> Pat {
    Pat::Lit(Lit::Int(n))
}

#[test]
fn inclusive_range_pattern() {
    assert_eq!(
        arm_pat("match age with | 0..=12 -> 1"),
        Pat::Range(Box::new(lit_pat(0)), Box::new(lit_pat(12))),
    );
}

#[test]
fn multiple_range_arms() {
    let e = parse_expr("match age with | 0..=12 -> 1 | 13..=19 -> 2 | _ -> 3")
        .expect("parses");
    if let Expr::Match(_, arms) = e {
        assert_eq!(arms.len(), 3);
        assert_eq!(arms[0].pat, Pat::Range(Box::new(lit_pat(0)), Box::new(lit_pat(12))));
        assert_eq!(arms[1].pat, Pat::Range(Box::new(lit_pat(13)), Box::new(lit_pat(19))));
        assert_eq!(arms[2].pat, Pat::Wildcard);
    } else {
        panic!("expected match");
    }
}
