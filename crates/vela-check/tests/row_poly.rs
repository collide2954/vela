use vela_check::{Type, check_expr, check_program};

fn t(src: &str) -> Type {
    check_expr(src).expect("type-checks")
}

#[test]
fn lambda_reads_field_from_closed_record() {
    assert_eq!(t("(fn r -> r.x) { x = 1 }"), Type::Int);
}

#[test]
fn lambda_reads_field_from_record_with_extra_field() {
    assert_eq!(t("(fn r -> r.x) { x = 1, y = 2 }"), Type::Int);
}

#[test]
fn lambda_reads_field_from_record_with_many_extras() {
    assert_eq!(
        t(r#"(fn r -> r.x) { a = 1, b = 2, x = 99, c = 3 }"#),
        Type::Int,
    );
}

#[test]
fn polymorphic_field_accessor_used_at_two_record_shapes() {
    let src = r#"let get_x = fn r -> r.x
let a = get_x { x = 1, y = 2 }
let b = get_x { x = "hi", z = 0 }
b"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::String);
}

#[test]
fn missing_field_is_still_an_error() {
    let e = check_expr("(fn r -> r.x) { y = 1 }").unwrap_err().message;
    assert!(e.contains("x"));
}

#[test]
fn two_field_reads_from_same_record() {
    let src = "let p = { x = 1, y = 2 }\np.x + p.y";
    assert_eq!(check_program(src).expect("type-checks"), Type::Int);
}
