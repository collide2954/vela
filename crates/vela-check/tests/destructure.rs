use vela_check::{Type, check_program};

#[test]
fn tuple_destructuring_binds_both() {
    let src = r#"let (a, b) = (1, "x")
b"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::String);
}

#[test]
fn tuple_destructuring_first_element() {
    let src = r#"let (a, _) = (1, "x")
a"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::Int);
}

#[test]
fn record_destructuring_punning() {
    let src = r#"let { x, y } = { x = 1, y = 2 }
x + y"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::Int);
}

#[test]
fn destructuring_type_mismatch_fails() {
    let src = r#"let (a, b) = 1
a"#;
    let e = check_program(src).unwrap_err().message;
    assert!(!e.is_empty());
}
