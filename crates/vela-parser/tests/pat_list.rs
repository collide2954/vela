use vela_parser::{Expr, ListPart, Lit, Pat, parse_expr};

fn p(src: &str) -> Expr {
    parse_expr(src).expect("parses")
}

fn arm_pat(src: &str) -> Pat {
    if let Expr::Match(_, arms) = p(src) {
        arms[0].pat.clone()
    } else {
        panic!("expected match");
    }
}

#[test]
fn empty_list_pattern() {
    assert_eq!(arm_pat("match xs with | [] -> 0"), Pat::List(vec![]),);
}

#[test]
fn single_element_list_pattern() {
    assert_eq!(
        arm_pat("match xs with | [x] -> x"),
        Pat::List(vec![ListPart::Pat(Pat::Var("x".into()))]),
    );
}

#[test]
fn multi_element_list_pattern() {
    assert_eq!(
        arm_pat("match xs with | [a, b, c] -> a"),
        Pat::List(vec![
            ListPart::Pat(Pat::Var("a".into())),
            ListPart::Pat(Pat::Var("b".into())),
            ListPart::Pat(Pat::Var("c".into())),
        ]),
    );
}

#[test]
fn head_tail_pattern() {
    assert_eq!(
        arm_pat("match xs with | [x, ..rest] -> x"),
        Pat::List(vec![
            ListPart::Pat(Pat::Var("x".into())),
            ListPart::Rest(Some("rest".into())),
        ]),
    );
}

#[test]
fn anonymous_rest() {
    assert_eq!(
        arm_pat("match xs with | [x, .._] -> x"),
        Pat::List(vec![
            ListPart::Pat(Pat::Var("x".into())),
            ListPart::Rest(None),
        ]),
    );
}

#[test]
fn rest_in_middle() {
    assert_eq!(
        arm_pat("match xs with | [first, ..middle, last] -> last"),
        Pat::List(vec![
            ListPart::Pat(Pat::Var("first".into())),
            ListPart::Rest(Some("middle".into())),
            ListPart::Pat(Pat::Var("last".into())),
        ]),
    );
}

#[test]
fn literal_in_list_pattern() {
    assert_eq!(
        arm_pat("match xs with | [0, x] -> x"),
        Pat::List(vec![
            ListPart::Pat(Pat::Lit(Lit::Int(0))),
            ListPart::Pat(Pat::Var("x".into())),
        ]),
    );
}
