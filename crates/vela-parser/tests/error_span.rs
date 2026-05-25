use vela_parser::parse_program;

#[test]
fn unexpected_token_carries_a_span() {
    let src = "let x = )\n";
    let err = parse_program(src).unwrap_err();
    assert!(err.span.is_some(), "expected error to have a span");
    let span = err.span.unwrap();
    assert!(span.start < src.len());
}

#[test]
fn missing_assign_in_let_has_span_near_let_name() {
    let src = "let x 5";
    let err = parse_program(src).unwrap_err();
    assert!(err.span.is_some());
}
