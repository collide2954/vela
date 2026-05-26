use vela_check::{Type, check_expr};

#[test]
fn float_of_int() {
    assert_eq!(
        check_expr("Float.of_int 1").expect("type-checks"),
        Type::Float
    );
}

#[test]
fn int_of_float() {
    assert_eq!(
        check_expr("Int.of_float 1.5").expect("type-checks"),
        Type::Int
    );
}

#[test]
fn float_to_string() {
    assert_eq!(
        check_expr("Float.to_string 3.14").expect("type-checks"),
        Type::String,
    );
}

#[test]
fn result_unwrap_int() {
    assert_eq!(
        check_expr("Result.unwrap (Ok 5)").expect("type-checks"),
        Type::Int,
    );
}

#[test]
fn option_unwrap_string() {
    assert_eq!(
        check_expr(r#"Option.unwrap (Some "abc")"#).expect("type-checks"),
        Type::String,
    );
}

#[test]
fn unknown_namespace_field_errors() {
    let e = check_expr("Float.no_such_thing 1").unwrap_err().message;
    assert!(e.contains("no_such_thing") || e.contains("field"));
}
