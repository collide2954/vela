use vela_check::{Type, check_expr};

#[test]
fn formula_with_identifiers() {
    assert_eq!(
        check_expr("y ~ x1 + x2").expect("type-checks"),
        Type::Formula,
    );
}

#[test]
fn formula_with_complex_rhs() {
    assert_eq!(
        check_expr("y ~ x1 * x2 + x3").expect("type-checks"),
        Type::Formula,
    );
}

#[test]
fn stream_unfold_yields_series() {
    let result = check_expr("Stream.unfold (fn s -> Some (s, s + 1)) 0").expect("type-checks");
    if let Type::Series(_) = result {
        return;
    }
    panic!("expected Series");
}
