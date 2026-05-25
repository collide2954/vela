use vela_check::{Type, TypeError, check_program};

fn t(src: &str) -> Type {
    check_program(src).expect("type-checks")
}

fn err(src: &str) -> TypeError {
    check_program(src).unwrap_err()
}

#[test]
fn simple_let_then_use() {
    assert_eq!(t("let x = 1\nx"), Type::Int);
}

#[test]
fn two_lets_then_sum() {
    assert_eq!(t("let x = 1\nlet y = 2\nx + y"), Type::Int);
}

#[test]
fn unbound_name_is_error() {
    let e = err("foo");
    assert!(e.message.contains("unbound") && e.message.contains("foo"));
}

#[test]
fn let_propagates_inferred_type() {
    assert_eq!(t("let s = \"hello\"\ns ++ \" world\""), Type::String);
}

#[test]
fn later_binding_shadows_earlier() {
    assert_eq!(t("let x = 1\nlet x = \"a\"\nx"), Type::String);
}

#[test]
fn arithmetic_uses_bindings() {
    assert_eq!(t("let a = 2\nlet b = 3\na * b + 1"), Type::Int);
}

#[test]
fn empty_program_is_unit() {
    assert_eq!(t(""), Type::Unit);
}

#[test]
fn trailing_let_yields_unit() {
    assert_eq!(t("let x = 1"), Type::Unit);
}
