use vela_check::{Type, check_expr, check_program};

fn t(src: &str) -> Type {
    check_expr(src).expect("type-checks")
}

#[test]
fn pipe_to_lambda() {
    assert_eq!(t("5 |> (fn x -> x + 1)"), Type::Int);
}

#[test]
fn pipe_to_named_function() {
    let src = "let inc = fn x -> x + 1\n5 |> inc";
    assert_eq!(check_program(src).expect("type-checks"), Type::Int);
}

#[test]
fn pipe_chains() {
    let src = "let inc = fn x -> x + 1\nlet double = fn x -> x * 2\n5 |> inc |> double";
    assert_eq!(check_program(src).expect("type-checks"), Type::Int);
}

#[test]
fn pipe_with_type_mismatch_fails() {
    let e = check_expr("\"abc\" |> (fn x -> x + 1)")
        .unwrap_err()
        .message;
    assert!(e.contains("Int") && e.contains("String"));
}

#[test]
fn pipe_returns_function_result_type() {
    let src = "let length = fn _ -> 0\n\"abc\" |> length";
    assert_eq!(check_program(src).expect("type-checks"), Type::Int);
}
