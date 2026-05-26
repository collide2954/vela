use vela_check::{Type, check_program};

fn t(src: &str) -> Type {
    check_program(src).expect("type-checks")
}

#[test]
fn identity_used_at_int() {
    assert_eq!(t("let id = fn x -> x\nid 5"), Type::Int);
}

#[test]
fn identity_used_at_string() {
    assert_eq!(
        t(r#"let id = fn x -> x
id "hi""#),
        Type::String
    );
}

#[test]
fn identity_used_at_two_types_in_one_program() {
    let src = r#"let id = fn x -> x
let a = id 5
let b = id "x"
b"#;
    assert_eq!(t(src), Type::String);
}

#[test]
fn const_function_specializes_independently() {
    let src = r#"let k = fn x y -> x
let a = k 1 "hi"
let b = k "yes" 3
b"#;
    assert_eq!(t(src), Type::String);
}

#[test]
fn polymorphic_used_then_unified_does_not_pollute_scheme() {
    let src = r#"let id = fn x -> x
let n = id 5
let s = id "abc"
s"#;
    assert_eq!(t(src), Type::String);
}
