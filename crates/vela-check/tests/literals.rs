use vela_check::{Type, check_expr};

fn t(src: &str) -> Type {
    check_expr(src).expect("type-checks")
}

#[test]
fn int_literal_is_int() {
    assert_eq!(t("42"), Type::Int);
}

#[test]
fn float_literal_is_float() {
    assert_eq!(t("3.14"), Type::Float);
}

#[test]
fn string_literal_is_string() {
    assert_eq!(t(r#""hello""#), Type::String);
}

#[test]
fn bool_literal_is_bool() {
    assert_eq!(t("true"), Type::Bool);
}

#[test]
fn unit_literal_is_unit() {
    assert_eq!(t("()"), Type::Unit);
}
