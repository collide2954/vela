use vela_check::{Type, check_expr, check_program};

#[test]
fn mean_of_float_series() {
    assert_eq!(
        check_expr("mean [1.0, 2.0, 3.0]").expect("type-checks"),
        Type::Float,
    );
}

#[test]
fn std_of_float_series() {
    assert_eq!(
        check_expr("std [1.0, 2.0, 3.0]").expect("type-checks"),
        Type::Float,
    );
}

#[test]
fn standardize_spec_example_type_checks() {
    let src = r#"let standardize (xs : [Float]) : [Float] =
    let m = mean xs
    let s = std xs
    map (fn x -> (x - m) / s) xs

standardize [1.0, 2.0, 3.0]"#;
    assert_eq!(
        check_program(src).expect("type-checks"),
        Type::Series(Box::new(Type::Float)),
    );
}

#[test]
fn string_length() {
    assert_eq!(
        check_expr(r#"String.length "abc""#).expect("type-checks"),
        Type::Int
    );
}

#[test]
fn string_concat() {
    assert_eq!(
        check_expr(r#"String.concat ["a", "b", "c"]"#).expect("type-checks"),
        Type::String,
    );
}

#[test]
fn dataframe_field_access_yields_series_of_option() {
    let src = r#"let df = {| x : [1, 2, 3] |}
df.x"#;
    let result = check_program(src).expect("type-checks");
    if let Type::Series(inner) = result
        && let Type::Option(_) = *inner
    {
        return;
    }
    panic!("expected Series[Option[_]]");
}
