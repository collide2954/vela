use vela_eval::run_tests;

#[test]
fn passing_tests() {
    let src = r#"tests =
    test "true is true" =
        assert true
    test "math" =
        assert (1 + 1 == 2)"#;
    let reports = run_tests(src).expect("no parse error");
    assert_eq!(reports.len(), 2);
    assert!(reports.iter().all(|r| r.passed));
}

#[test]
fn failing_test_is_reported() {
    let src = r#"tests =
    test "always fails" =
        assert false"#;
    let reports = run_tests(src).expect("no parse error");
    assert_eq!(reports.len(), 1);
    assert!(!reports[0].passed);
}

#[test]
fn mixed_pass_and_fail() {
    let src = r#"tests =
    test "ok" =
        assert true
    test "nope" =
        assert (1 == 2)"#;
    let reports = run_tests(src).expect("no parse error");
    assert_eq!(reports.len(), 2);
    assert!(reports[0].passed);
    assert!(!reports[1].passed);
}

#[test]
fn test_can_use_module_definitions() {
    let src = r#"let double x = x * 2

tests =
    test "doubles" =
        assert (double 5 == 10)"#;
    let reports = run_tests(src).expect("no parse error");
    assert_eq!(reports.len(), 1);
    assert!(reports[0].passed);
}
