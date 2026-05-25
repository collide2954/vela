use vela_check::{Type, TypeError, check_expr};

fn t(src: &str) -> Type {
    check_expr(src).expect("type-checks")
}

fn err(src: &str) -> TypeError {
    check_expr(src).unwrap_err()
}

#[test]
fn int_addition_is_int() {
    assert_eq!(t("1 + 2"), Type::Int);
}

#[test]
fn float_addition_is_float() {
    assert_eq!(t("1.0 + 2.0"), Type::Float);
}

#[test]
fn int_and_float_is_a_type_error() {
    let e = err("1 + 1.0");
    assert!(e.message.contains("Int") && e.message.contains("Float"));
}

#[test]
fn int_multiplication_chain() {
    assert_eq!(t("2 * 3 + 4"), Type::Int);
}

#[test]
fn unary_negation_preserves_type() {
    assert_eq!(t("-(1 + 2)"), Type::Int);
    assert_eq!(t("-3.14"), Type::Float);
}

#[test]
fn equality_returns_bool() {
    assert_eq!(t("1 == 2"), Type::Bool);
    assert_eq!(t(r#""a" == "b""#), Type::Bool);
}

#[test]
fn comparison_returns_bool() {
    assert_eq!(t("1 < 2"), Type::Bool);
    assert_eq!(t("1.0 >= 2.0"), Type::Bool);
}

#[test]
fn equality_across_types_is_error() {
    let e = err(r#"1 == "a""#);
    assert!(e.message.contains("Int") && e.message.contains("String"));
}

#[test]
fn boolean_and_is_bool() {
    assert_eq!(t("true and false"), Type::Bool);
}

#[test]
fn boolean_or_is_bool() {
    assert_eq!(t("true or false"), Type::Bool);
}

#[test]
fn not_returns_bool() {
    assert_eq!(t("not true"), Type::Bool);
}

#[test]
fn not_on_non_bool_is_error() {
    let e = err("not 1");
    assert!(e.message.contains("Bool"));
}

#[test]
fn string_concat_is_string() {
    assert_eq!(t(r#""hello" ++ " world""#), Type::String);
}
