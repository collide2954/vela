use vela_check::{Type, check_program};

#[test]
fn trait_method_registered_as_polymorphic() {
    let src = r#"trait Show t =
    fn show (x : t) : String

show 3.14"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::String);
}

#[test]
fn trait_method_works_at_different_types() {
    let src = r#"trait Show t =
    fn show (x : t) : String

let a = show 1
let b = show "hi"
a"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::String);
}

#[test]
fn trait_method_unifies_param_with_arg() {
    let src = r#"trait Eq t =
    fn eq (a : t) (b : t) : Bool

eq 1 2"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::Bool);
}

#[test]
fn trait_method_mismatched_args_fails() {
    let src = r#"trait Eq t =
    fn eq (a : t) (b : t) : Bool

eq 1 "x""#;
    let e = check_program(src).unwrap_err().message;
    assert!(e.contains("Int") && e.contains("String"));
}

#[test]
fn impl_body_type_checks_against_trait_signature() {
    let src = r#"trait Show t =
    fn show (x : t) : String

impl Show Float =
    fn show x = Float.to_string x

show 3.14"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::String);
}
