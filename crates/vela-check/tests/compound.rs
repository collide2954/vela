use vela_check::{Type, check_expr};

fn t(src: &str) -> Type {
    check_expr(src).expect("type-checks")
}

fn err(src: &str) -> String {
    check_expr(src).unwrap_err().message
}

fn series(inner: Type) -> Type {
    Type::Series(Box::new(inner))
}

#[test]
fn integer_series() {
    assert_eq!(t("[1, 2, 3]"), series(Type::Int));
}

#[test]
fn float_series() {
    assert_eq!(t("[1.0, 2.0]"), series(Type::Float));
}

#[test]
fn string_series() {
    assert_eq!(t(r#"["a", "b"]"#), series(Type::String));
}

#[test]
fn heterogeneous_series_is_error() {
    let e = err(r#"[1, "a"]"#);
    assert!(e.contains("Int") && e.contains("String"));
}

#[test]
fn empty_series_is_polymorphic_series() {
    let result = t("[]");
    assert!(matches!(result, Type::Series(_)));
}

#[test]
fn two_element_tuple() {
    assert_eq!(t(r#"(1, "a")"#), Type::Tuple(vec![Type::Int, Type::String]));
}

#[test]
fn three_element_tuple() {
    assert_eq!(
        t("(1, 2.0, true)"),
        Type::Tuple(vec![Type::Int, Type::Float, Type::Bool]),
    );
}

#[test]
fn record_literal() {
    assert_eq!(
        t("{ x = 1, y = 2.0 }"),
        Type::Record(vec![("x".into(), Type::Int), ("y".into(), Type::Float)]),
    );
}

#[test]
fn record_field_access() {
    assert_eq!(t("{ x = 1, y = 2.0 }.x"), Type::Int);
    assert_eq!(t("{ x = 1, y = 2.0 }.y"), Type::Float);
}

#[test]
fn field_access_missing_field_is_error() {
    let e = err("{ x = 1 }.z");
    assert!(e.contains("z"));
}

#[test]
fn symbol_is_symbol() {
    assert_eq!(t(":species"), Type::Symbol);
}

#[test]
fn series_of_records() {
    assert_eq!(
        t("[{ x = 1 }, { x = 2 }]"),
        series(Type::Record(vec![("x".into(), Type::Int)])),
    );
}
