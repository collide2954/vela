use vela_parser::{Param, Stmt, TestCase, Ty, parse_stmt};

fn s(src: &str) -> Stmt {
    parse_stmt(src).expect("parses")
}

fn con(n: &str) -> Ty {
    Ty::Con(n.into())
}

#[test]
fn single_test_case() {
    let stmt = s(r#"tests =
    test "easy" = 1"#);
    if let Stmt::Tests(cases) = stmt {
        assert_eq!(cases.len(), 1);
        if let TestCase::Test { name, .. } = &cases[0] {
            assert_eq!(name, "easy");
        } else {
            panic!("expected test case");
        }
    } else {
        panic!("expected tests block");
    }
}

#[test]
fn multiple_test_cases() {
    let stmt = s(r#"tests =
    test "a" = 1
    test "b" = 2
    test "c" = 3"#);
    if let Stmt::Tests(cases) = stmt {
        assert_eq!(cases.len(), 3);
    } else {
        panic!("expected tests block");
    }
}

#[test]
fn prop_case_with_typed_param_and_guard() {
    let stmt = s(r#"tests =
    prop "positive" (n : Int) when n > 0 = n + 1 > n"#);
    if let Stmt::Tests(cases) = stmt {
        assert_eq!(cases.len(), 1);
        if let TestCase::Prop { name, params, guard, .. } = &cases[0] {
            assert_eq!(name, "positive");
            assert_eq!(params.len(), 1);
            assert_eq!(params[0], Param { pat: vela_parser::Pat::Var("n".into()), ty: Some(con("Int")) });
            assert!(guard.is_some());
        } else {
            panic!("expected prop case");
        }
    } else {
        panic!("expected tests block");
    }
}

#[test]
fn mixed_test_and_prop_cases() {
    let stmt = s(r#"tests =
    test "literal" = 42
    prop "any" (x : Int) = x + 0"#);
    if let Stmt::Tests(cases) = stmt {
        assert_eq!(cases.len(), 2);
        assert!(matches!(cases[0], TestCase::Test { .. }));
        assert!(matches!(cases[1], TestCase::Prop { .. }));
    } else {
        panic!("expected tests block");
    }
}
