use vela_check::{Type, check_expr, check_program};

fn fn_t(a: Type, b: Type) -> Type {
    Type::Fn(Box::new(a), Box::new(b))
}

#[test]
fn identity_lambda_is_polymorphic_within_an_expression() {
    let t = check_expr("fn x -> x").expect("type-checks");
    // T -> T for some fresh T; check by structure
    match t {
        Type::Fn(a, b) => assert_eq!(a, b),
        other => panic!("expected fn type, got {other:?}"),
    }
}

#[test]
fn lambda_body_constrains_param_type() {
    assert_eq!(
        check_expr("fn x -> x + 1").expect("type-checks"),
        fn_t(Type::Int, Type::Int),
    );
}

#[test]
fn two_param_curried_lambda() {
    assert_eq!(
        check_expr("fn x y -> x + y + 1").expect("type-checks"),
        fn_t(Type::Int, fn_t(Type::Int, Type::Int)),
    );
}

#[test]
fn applying_identity_to_int_yields_int() {
    assert_eq!(check_expr("(fn x -> x) 5").expect("type-checks"), Type::Int,);
}

#[test]
fn applying_curried_lambda() {
    assert_eq!(
        check_expr("(fn x y -> x + y) 2 3").expect("type-checks"),
        Type::Int,
    );
}

#[test]
fn let_function_and_call() {
    assert_eq!(
        check_program("let f = fn x -> x + 1\nf 2").expect("type-checks"),
        Type::Int,
    );
}

#[test]
fn applying_to_wrong_type_fails() {
    let e = check_expr(r#"(fn x -> x + 1) "hello""#).unwrap_err();
    assert!(e.message.contains("Int") && e.message.contains("String"));
}

#[test]
fn higher_order_function() {
    // applies f to 5
    assert_eq!(
        check_expr("(fn f -> f 5) (fn x -> x + 1)").expect("type-checks"),
        Type::Int,
    );
}
