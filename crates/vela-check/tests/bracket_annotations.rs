use vela_check::{Type, check_program};

#[test]
fn option_int_annotation() {
    let src = "let m : Option[Int] = Some 5\nm";
    assert_eq!(
        check_program(src).expect("type-checks"),
        Type::Option(Box::new(Type::Int)),
    );
}

#[test]
fn result_annotation() {
    let src = r#"let r : Result[Int, String] = Ok 5
r"#;
    let result = check_program(src).expect("type-checks");
    if let Type::Result(a, e) = result {
        assert_eq!(*a, Type::Int);
        assert_eq!(*e, Type::String);
    } else {
        panic!("expected Result");
    }
}

#[test]
fn option_string_annotation_rejects_wrong_value() {
    let src = r#"let m : Option[String] = Some 5
m"#;
    let e = check_program(src).unwrap_err().message;
    assert!(e.contains("Int") && e.contains("String"));
}

#[test]
fn user_named_type_via_bracket() {
    let src = r#"type Box 'a = | Box 'a
let b : Box[Int] = Box 5
b"#;
    let result = check_program(src).expect("type-checks");
    assert_eq!(result, Type::Named("Box".into(), vec![Type::Int]));
}
