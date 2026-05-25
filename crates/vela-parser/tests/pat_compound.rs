use vela_parser::{Expr, Pat, parse_expr};

fn arm_pat(src: &str) -> Pat {
    if let Expr::Match(_, arms) = parse_expr(src).expect("parses") {
        arms[0].pat.clone()
    } else {
        panic!("expected match");
    }
}

fn var(s: &str) -> Pat {
    Pat::Var(s.into())
}
fn cons(n: &str, args: Vec<Pat>) -> Pat {
    Pat::Cons(n.into(), args)
}

#[test]
fn tuple_pattern_two_elements() {
    assert_eq!(
        arm_pat("match p with | (a, b) -> 0"),
        Pat::Tuple(vec![var("a"), var("b")]),
    );
}

#[test]
fn tuple_pattern_three_elements() {
    assert_eq!(
        arm_pat("match p with | (a, b, c) -> 0"),
        Pat::Tuple(vec![var("a"), var("b"), var("c")]),
    );
}

#[test]
fn parens_around_single_pat_unwrap() {
    assert_eq!(arm_pat("match p with | (x) -> 0"), var("x"));
}

#[test]
fn record_pat_punning() {
    assert_eq!(
        arm_pat("match p with | { x, y } -> 0"),
        Pat::Record(vec![("x".into(), var("x")), ("y".into(), var("y"))]),
    );
}

#[test]
fn record_pat_explicit_bindings() {
    assert_eq!(
        arm_pat("match p with | { x = a, y = b } -> 0"),
        Pat::Record(vec![("x".into(), var("a")), ("y".into(), var("b"))]),
    );
}

#[test]
fn record_pat_inside_constructor() {
    assert_eq!(
        arm_pat("match s with | Rect { width = w, height = h } -> 0"),
        cons(
            "Rect",
            vec![Pat::Record(vec![
                ("width".into(), var("w")),
                ("height".into(), var("h")),
            ])],
        ),
    );
}

#[test]
fn tuple_inside_constructor() {
    assert_eq!(
        arm_pat("match p with | Pair (a, b) -> 0"),
        cons("Pair", vec![Pat::Tuple(vec![var("a"), var("b")])]),
    );
}
