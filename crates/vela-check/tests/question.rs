use vela_check::{Type, check_expr};

fn t(src: &str) -> Type {
    check_expr(src).expect("type-checks")
}

#[test]
fn question_on_ok_extracts_inner() {
    assert_eq!(t("(Ok 5)?"), Type::Int);
}

#[test]
fn question_on_result_string() {
    assert_eq!(t(r#"(Ok "hello")?"#), Type::String);
}

#[test]
fn question_on_non_result_fails() {
    let e = check_expr("5?").unwrap_err().message;
    assert!(e.contains("Result") || e.contains("Int"));
}

#[test]
fn question_in_arithmetic() {
    assert_eq!(t("(Ok 5)? + 1"), Type::Int);
}
