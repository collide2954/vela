use vela_check::{check_expr, check_program};

fn ok(src: &str) {
    check_expr(src).expect("should type-check");
}

fn err(src: &str) -> String {
    check_expr(src).unwrap_err().message
}

#[test]
fn wildcard_makes_int_match_exhaustive() {
    ok("match 1 with | 0 -> 0 | _ -> 1");
}

#[test]
fn var_makes_int_match_exhaustive() {
    ok("match 1 with | n -> n");
}

#[test]
fn int_match_without_wildcard_is_non_exhaustive() {
    let e = err("match 1 with | 0 -> 0 | 1 -> 1");
    assert!(e.contains("non-exhaustive"));
}

#[test]
fn bool_match_with_both_is_exhaustive() {
    ok("match true with | true -> 1 | false -> 0");
}

#[test]
fn bool_match_missing_false_is_non_exhaustive() {
    let e = err("match true with | true -> 1");
    assert!(e.contains("non-exhaustive") && e.contains("false"));
}

#[test]
fn option_match_with_both_constructors() {
    ok("match Some 5 with | None -> 0 | Some x -> x");
}

#[test]
fn option_match_missing_none_is_non_exhaustive() {
    let e = err("match Some 5 with | Some x -> x");
    assert!(e.contains("non-exhaustive") && e.contains("None"));
}

#[test]
fn result_match_with_both() {
    ok(r#"match Ok 5 with | Ok v -> v | Err _ -> 0"#);
}

#[test]
fn result_match_missing_err_is_non_exhaustive() {
    let e = err("match Ok 5 with | Ok v -> v");
    assert!(e.contains("non-exhaustive") && e.contains("Err"));
}

#[test]
fn user_sum_exhaustive() {
    let src = r#"type Color = | Red | Blue | Green
match Red with | Red -> 1 | Blue -> 2 | Green -> 3"#;
    check_program(src).expect("should type-check");
}

#[test]
fn user_sum_missing_one_variant() {
    let src = r#"type Color = | Red | Blue | Green
match Red with | Red -> 1 | Blue -> 2"#;
    let e = check_program(src).unwrap_err().message;
    assert!(e.contains("non-exhaustive") && e.contains("Green"));
}

#[test]
fn or_pattern_covers_multiple_variants() {
    let src = r#"type Color = | Red | Blue | Green
match Red with | Red | Blue -> 1 | Green -> 2"#;
    check_program(src).expect("should type-check");
}

#[test]
fn as_binding_does_not_break_exhaustiveness() {
    ok("match true with | true as t -> 1 | false -> 0");
}
