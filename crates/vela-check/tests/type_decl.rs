use vela_check::{Type, check_program};

fn t(src: &str) -> Type {
    check_program(src).expect("type-checks")
}

fn err(src: &str) -> String {
    check_program(src).unwrap_err().message
}

fn named(name: &str, args: Vec<Type>) -> Type {
    Type::Named(name.into(), args)
}

#[test]
fn nullary_constructor_value() {
    let src = "type Color = | Red | Blue | Green\nRed";
    assert_eq!(t(src), named("Color", vec![]));
}

#[test]
fn unary_constructor_returns_named() {
    let src = "type Shape = | Circle Float | Square Float\nCircle 3.0";
    assert_eq!(t(src), named("Shape", vec![]));
}

#[test]
fn match_on_user_sum() {
    let src = r#"type Shape = | Circle Float | Square Float
match Circle 3.0 with
| Circle r -> r
| Square s -> s"#;
    assert_eq!(t(src), Type::Float);
}

#[test]
fn parametric_type_constructor() {
    let src = "type Box 'a = | Box 'a\nBox 5";
    assert_eq!(t(src), named("Box", vec![Type::Int]));
}

#[test]
fn parametric_match() {
    let src = r#"type Box 'a = | Box 'a
match Box 5 with
| Box x -> x"#;
    assert_eq!(t(src), Type::Int);
}

#[test]
fn wrong_argument_to_constructor_fails() {
    let src = r#"type Shape = | Circle Float
Circle "abc""#;
    let e = err(src);
    assert!(e.contains("Float") && e.contains("String"));
}

#[test]
fn constructors_share_type_through_match() {
    let src = r#"type Maybe 'a = | Nothing | Just 'a
match Just 5 with
| Nothing -> 0
| Just x -> x"#;
    assert_eq!(t(src), Type::Int);
}
