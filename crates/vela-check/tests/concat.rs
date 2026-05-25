use vela_check::{Type, check_expr};

fn t(src: &str) -> Type {
    check_expr(src).expect("type-checks")
}

#[test]
fn string_concat() {
    assert_eq!(t(r#""a" ++ "b""#), Type::String);
}

#[test]
fn series_concat_yields_series_of_same_inner_type() {
    assert_eq!(t("[1, 2] ++ [3, 4]"), Type::Series(Box::new(Type::Int)));
}

#[test]
fn concat_requires_matching_inner_types() {
    let e = check_expr(r#"[1] ++ ["a"]"#).unwrap_err().message;
    assert!(e.contains("Int") && e.contains("String"));
}

#[test]
fn cannot_concat_int_with_int() {
    let e = check_expr("1 ++ 2").unwrap_err().message;
    assert!(e.contains("Int"));
}
