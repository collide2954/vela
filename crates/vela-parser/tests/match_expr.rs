use vela_parser::{Expr, Lit, MatchArm, Pat, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn lit(n: i64) -> Expr {
    Expr::Lit(Lit::Int(n))
}
fn var(s: &str) -> Expr {
    Expr::Var(s.into())
}
fn lit_pat(n: i64) -> Pat {
    Pat::Lit(Lit::Int(n))
}
fn var_pat(s: &str) -> Pat {
    Pat::Var(s.into())
}
fn cons(name: &str, args: Vec<Pat>) -> Pat {
    Pat::Cons(name.into(), args)
}
fn matchx(scrut: Expr, arms: Vec<MatchArm>) -> Expr {
    Expr::Match(Box::new(scrut), arms)
}
fn arm(pat: Pat, body: Expr) -> MatchArm {
    MatchArm { pat, guard: None, body }
}

#[test]
fn match_int_literal() {
    assert_eq!(
        p("match x with | 0 -> 1 | _ -> 2"),
        matchx(
            var("x"),
            vec![arm(lit_pat(0), lit(1)), arm(Pat::Wildcard, lit(2))],
        ),
    );
}

#[test]
fn match_result() {
    assert_eq!(
        p("match r with | Ok v -> v | Err e -> e"),
        matchx(
            var("r"),
            vec![
                arm(cons("Ok", vec![var_pat("v")]), var("v")),
                arm(cons("Err", vec![var_pat("e")]), var("e")),
            ],
        ),
    );
}

#[test]
fn match_wildcard_only() {
    assert_eq!(
        p("match x with | _ -> 0"),
        matchx(var("x"), vec![arm(Pat::Wildcard, lit(0))]),
    );
}

#[test]
fn match_with_string_pattern() {
    assert_eq!(
        p(r#"match s with | "yes" -> 1 | _ -> 0"#),
        matchx(
            var("s"),
            vec![
                arm(Pat::Lit(Lit::Str("yes".into())), lit(1)),
                arm(Pat::Wildcard, lit(0)),
            ],
        ),
    );
}

#[test]
fn nullary_constructor_pattern() {
    assert_eq!(
        p("match c with | None -> 0 | Some x -> x"),
        matchx(
            var("c"),
            vec![
                arm(cons("None", vec![]), lit(0)),
                arm(cons("Some", vec![var_pat("x")]), var("x")),
            ],
        ),
    );
}
