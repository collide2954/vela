use vela_check::{Type, check_expr};

// `?` requires an enclosing function returning Result, so each test
// wraps the use in a lambda whose body returns Ok of something.

fn t(src: &str) -> Type {
    check_expr(src).expect("type-checks")
}

#[test]
fn question_on_ok_extracts_inner_inside_lambda() {
    let result = t("fn _ -> Ok ((Ok 5)?)");
    if let Type::Fn(_, ret) = result
        && let Type::Result(a, _) = *ret
    {
        assert_eq!(*a, Type::Int);
        return;
    }
    panic!("expected Fn returning Result Int");
}

#[test]
fn question_on_non_result_fails_in_lambda() {
    let e = check_expr("fn _ -> Ok (5?)").unwrap_err().message;
    assert!(e.contains("Result") || e.contains("Int"));
}

#[test]
fn question_at_top_level_is_error() {
    let e = check_expr("(Ok 5)?").unwrap_err().message;
    assert!(e.contains("?") || e.contains("Result"));
}
