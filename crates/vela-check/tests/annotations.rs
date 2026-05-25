use vela_check::{Type, check_program};

fn t(src: &str) -> Type {
    check_program(src).expect("type-checks")
}

fn err(src: &str) -> String {
    check_program(src).unwrap_err().message
}

#[test]
fn let_with_matching_annotation() {
    assert_eq!(t("let x : Int = 1\nx"), Type::Int);
}

#[test]
fn let_with_mismatched_annotation_fails() {
    let e = err("let x : Float = 1\nx");
    assert!(e.contains("Int") && e.contains("Float"));
}

#[test]
fn typed_param_constrains_inference() {
    let src = r#"let id (x : Int) = x
id 5"#;
    assert_eq!(t(src), Type::Int);
}

#[test]
fn typed_param_rejects_wrong_arg_type() {
    let src = r#"let id (x : Int) = x
id "a""#;
    let e = check_program(src).unwrap_err().message;
    assert!(e.contains("Int") && e.contains("String"));
}

#[test]
fn return_type_annotation_must_match_body() {
    let e = err(r#"let f (x : Int) : String = x"#);
    assert!(e.contains("Int") && e.contains("String"));
}

#[test]
fn polymorphic_signature_with_type_variable() {
    let src = r#"let id (x : 'a) : 'a = x
id 5"#;
    assert_eq!(t(src), Type::Int);
}

#[test]
fn polymorphic_signature_usable_at_two_types() {
    let src = r#"let id (x : 'a) : 'a = x
let a = id 5
let b = id "a"
b"#;
    assert_eq!(t(src), Type::String);
}

#[test]
fn var_with_annotation() {
    assert_eq!(t("var x : Int = 1\nx"), Type::Int);
}

#[test]
fn series_type_annotation() {
    let src = r#"let xs : [Int] = [1, 2, 3]
xs"#;
    assert_eq!(t(src), Type::Series(Box::new(Type::Int)));
}
