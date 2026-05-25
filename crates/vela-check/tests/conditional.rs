use vela_check::{Type, check_expr};

fn t(src: &str) -> Type {
    check_expr(src).expect("type-checks")
}

fn err(src: &str) -> String {
    check_expr(src).unwrap_err().message
}

#[test]
fn if_returns_branch_type() {
    assert_eq!(t("if true then 1 else 2"), Type::Int);
}

#[test]
fn if_string_branches() {
    assert_eq!(t(r#"if true then "yes" else "no""#), Type::String);
}

#[test]
fn if_branches_must_match() {
    let e = err(r#"if true then 1 else "a""#);
    assert!(e.contains("Int") && e.contains("String"));
}

#[test]
fn if_condition_must_be_bool() {
    let e = err("if 1 then 2 else 3");
    assert!(e.contains("Int") && e.contains("Bool"));
}

#[test]
fn match_on_int_with_wildcard() {
    assert_eq!(
        t(r#"match 1 with | 0 -> "zero" | _ -> "other""#),
        Type::String,
    );
}

#[test]
fn match_binds_var_in_arm_body() {
    assert_eq!(t("match 1 with | n -> n + 1"), Type::Int);
}

#[test]
fn match_arms_must_have_same_body_type() {
    let e = err(r#"match 1 with | 0 -> "zero" | _ -> 1"#);
    assert!(e.contains("String") && e.contains("Int"));
}

#[test]
fn match_pattern_type_must_match_scrutinee() {
    let e = err(r#"match 1 with | "a" -> 0 | _ -> 1"#);
    assert!(e.contains("Int") && e.contains("String"));
}

#[test]
fn match_on_bool() {
    assert_eq!(t("match true with | true -> 1 | false -> 0"), Type::Int);
}

#[test]
fn match_guard_must_be_bool() {
    let e = err("match 1 with | n when n + 1 -> 0 | _ -> 1");
    assert!(e.contains("Int") && e.contains("Bool"));
}

#[test]
fn match_with_valid_guard() {
    assert_eq!(t("match 1 with | n when n > 0 -> n | _ -> 0"), Type::Int);
}
