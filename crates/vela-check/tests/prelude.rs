use vela_check::{Type, check_expr, check_program};

#[test]
fn length_of_int_series() {
    assert_eq!(
        check_expr("length [1, 2, 3]").expect("type-checks"),
        Type::Int
    );
}

#[test]
fn map_inc_over_int_series() {
    let result = check_expr("map (fn x -> x + 1) [1, 2, 3]").expect("type-checks");
    assert_eq!(result, Type::Series(Box::new(Type::Int)));
}

#[test]
fn map_str_length_over_string_series() {
    let result = check_expr(r#"map (fn _ -> 0) ["a", "b"]"#).expect("type-checks");
    assert_eq!(result, Type::Series(Box::new(Type::Int)));
}

#[test]
fn filter_positive_ints() {
    let result = check_expr("filter (fn x -> x > 0) [1, -1, 2]").expect("type-checks");
    assert_eq!(result, Type::Series(Box::new(Type::Int)));
}

#[test]
fn sum_of_int_series() {
    assert_eq!(check_expr("sum [1, 2, 3]").expect("type-checks"), Type::Int);
}

#[test]
fn println_returns_unit() {
    assert_eq!(check_expr("println 42").expect("type-checks"), Type::Unit);
}

#[test]
fn read_file_returns_result_string() {
    let result = check_expr(r#"read_file "x.csv""#).expect("type-checks");
    if let Type::Result(a, _) = result {
        assert_eq!(*a, Type::String);
    } else {
        panic!("expected Result<String, _>");
    }
}

#[test]
fn realistic_pipeline_with_prelude() {
    let src = r#"let total =
    [1, 2, 3, 4, 5]
    |> filter (fn x -> x > 2)
    |> sum
total"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::Int);
}
