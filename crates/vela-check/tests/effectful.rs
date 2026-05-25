use vela_check::{Type, check_program};

fn t(src: &str) -> Type {
    check_program(src).expect("type-checks")
}

fn err(src: &str) -> String {
    check_program(src).unwrap_err().message
}

#[test]
fn for_loop_unit_body() {
    assert_eq!(t("for x in [1, 2, 3]:\n    ()"), Type::Unit);
}

#[test]
fn for_loop_with_non_unit_body_fails() {
    let e = err("for x in [1, 2, 3]:\n    x + 1");
    assert!(e.contains("Int") && e.contains("()"));
}

#[test]
fn for_loop_over_strings() {
    let src = r#"for x in ["a", "b"]:
    ()"#;
    assert_eq!(t(src), Type::Unit);
}

#[test]
fn for_loop_iter_must_be_series() {
    let e = err("for x in 1:\n    ()");
    assert!(e.contains("Int") || e.contains("Series"));
}

#[test]
fn mutation_with_matching_type() {
    let src = "var x = 1\nx <- 2\nx";
    assert_eq!(t(src), Type::Int);
}

#[test]
fn mutation_type_mismatch_fails() {
    let e = err(r#"var x = 1
x <- "a""#);
    assert!(e.contains("Int") && e.contains("String"));
}

#[test]
fn mutation_in_for_loop() {
    let src = r#"var total = 0
for x in [1, 2, 3]:
    total <- total + x
total"#;
    assert_eq!(t(src), Type::Int);
}
