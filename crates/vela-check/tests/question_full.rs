use vela_check::{Type, check_expr, check_program};

fn err(src: &str) -> String {
    check_expr(src).unwrap_err().message
}

#[test]
fn question_at_top_level_is_error() {
    let e = err("(Ok 5)?");
    assert!(e.contains("?") || e.contains("Result"));
}

#[test]
fn question_in_lambda_returning_int_is_error() {
    // Lambda body uses ? which forces Result return, but body's last
    // expression is Int — contradiction.
    let e = err("fn x -> (Ok 5)?");
    assert!(!e.is_empty());
}

#[test]
fn question_in_lambda_returning_ok_works() {
    let result = check_expr("fn x -> Ok ((Ok 5)?)").expect("type-checks");
    // 'a -> Result<Int, e>
    if let Type::Fn(_, ret) = result {
        if let Type::Result(a, _) = *ret {
            assert_eq!(*a, Type::Int);
        } else {
            panic!("expected Result return type");
        }
    } else {
        panic!("expected Fn");
    }
}

#[test]
fn question_propagates_error_type() {
    let src = r#"let load p =
    let a = (Ok 1)?
    let b = (Ok 2)?
    Ok (a + b)
load 0"#;
    let result = check_program(src);
    assert!(result.is_ok(), "should type-check: {:?}", result);
}

#[test]
fn let_function_with_question_returning_result() {
    let src = r#"let f x = Ok ((Ok x)?)
f 5"#;
    let result = check_program(src).expect("type-checks");
    if let Type::Result(a, _) = result {
        assert_eq!(*a, Type::Int);
    } else {
        panic!("expected Result");
    }
}
