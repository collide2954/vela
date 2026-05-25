use vela_check::{Type, check_program};

#[test]
fn realistic_program_with_types_match_and_records() {
    let src = r#"type Shape =
    | Circle Float
    | Square Float
    | Rect { width : Float, height : Float }

let area shape =
    match shape with
    | Circle r -> r * r
    | Square s -> s * s
    | Rect r -> r.width * r.height

area (Circle 3.0)"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::Float);
}

#[test]
fn realistic_program_with_option_propagation() {
    let src = r#"type Maybe 'a =
    | Nothing
    | Just 'a

let unwrap_or m default =
    match m with
    | Nothing -> default
    | Just x -> x

unwrap_or (Just 5) 0"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::Int);
}

#[test]
fn polymorphic_field_extractor() {
    let src = r#"let get_name r = r.name
let alice = { name = "Alice", age = 30 }
let pet = { name = "Rex", species = "dog" }
let names = [get_name alice, get_name pet]
names"#;
    assert_eq!(
        check_program(src).expect("type-checks"),
        Type::Series(Box::new(Type::String)),
    );
}

#[test]
fn pipeline_with_lambdas() {
    let src = r#"let inc = fn x -> x + 1
let double = fn x -> x * 2
5 |> inc |> double"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::Int);
}

#[test]
fn import_then_use_is_unit_for_import_stmt() {
    let src = r#"import std.data

let id x = x
id 5"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::Int);
}

#[test]
fn trait_decl_is_accepted_even_if_not_yet_resolved() {
    let src = r#"trait Show t =
    fn show (x : t) : String

let pi = 3.14
pi"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::Float);
}
