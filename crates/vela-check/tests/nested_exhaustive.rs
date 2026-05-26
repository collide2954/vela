use vela_check::check_expr;

fn ok(src: &str) {
    check_expr(src).expect("type-checks");
}

fn err(src: &str) -> String {
    check_expr(src).err().expect("error").message
}

#[test]
fn nested_option_in_option_full_cover() {
    ok("match Some (Some 1) with | None -> 0 | Some None -> 0 | Some (Some n) -> n");
}

#[test]
fn nested_option_in_option_missing_inner_some() {
    let e = err("match Some (Some 1) with | None -> 0 | Some None -> 0");
    assert!(e.contains("non-exhaustive"));
}

#[test]
fn nested_result_inside_option_full_cover() {
    ok("match Some (Ok 1) with | None -> 0 | Some (Ok n) -> n | Some (Err _) -> 0");
}

#[test]
fn nested_result_inside_option_missing_err() {
    let e = err("match Some (Ok 1) with | None -> 0 | Some (Ok n) -> n");
    assert!(e.contains("non-exhaustive"));
}

#[test]
fn tuple_of_bools_with_wildcard_arm() {
    ok("match (true, false) with | (true, _) -> 0 | (false, _) -> 1");
}
