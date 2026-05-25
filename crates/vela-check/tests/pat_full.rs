use vela_check::{Type, check_expr, check_program};

#[test]
fn tuple_pattern_in_match() {
    let src = r#"match (1, "a") with
| (n, s) -> n"#;
    assert_eq!(check_expr(src).expect("type-checks"), Type::Int);
}

#[test]
fn record_pattern_in_match() {
    let src = r#"match { x = 1, y = 2 } with
| { x = a, y = b } -> a + b"#;
    assert_eq!(check_expr(src).expect("type-checks"), Type::Int);
}

#[test]
fn record_pattern_with_punning() {
    let src = r#"match { x = 1, y = 2 } with
| { x, y } -> x + y"#;
    assert_eq!(check_expr(src).expect("type-checks"), Type::Int);
}

#[test]
fn record_pattern_inside_cons() {
    let src = r#"type Shape = | Rect { width : Float, height : Float }
match Rect { width = 3.0, height = 4.0 } with
| Rect { width = w, height = h } -> w * h"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::Float);
}

#[test]
fn list_pattern_head_tail_with_wildcard() {
    let src = r#"match [1, 2, 3] with
| [x, ..rest] -> x
| _ -> 0"#;
    assert_eq!(check_expr(src).expect("type-checks"), Type::Int);
}

#[test]
fn rest_only_pattern_is_absorbing() {
    let src = r#"match [1, 2, 3] with
| [..xs] -> 1"#;
    assert_eq!(check_expr(src).expect("type-checks"), Type::Int);
}

#[test]
fn list_pattern_destructures_anonymous_rest() {
    let src = r#"match [1, 2, 3] with
| [x, .._] -> x
| _ -> 0"#;
    assert_eq!(check_expr(src).expect("type-checks"), Type::Int);
}

#[test]
fn range_pattern_for_int() {
    let src = r#"match 5 with
| 0..=10 -> 1
| _ -> 0"#;
    assert_eq!(check_expr(src).expect("type-checks"), Type::Int);
}

#[test]
fn area_example_from_spec_with_record_variant() {
    let src = r#"type Shape =
    | Circle Float
    | Square Float
    | Rect { width : Float, height : Float }

let area shape =
    match shape with
    | Circle r -> r * r
    | Square s -> s * s
    | Rect { width = w, height = h } -> w * h

area (Rect { width = 3.0, height = 4.0 })"#;
    assert_eq!(check_program(src).expect("type-checks"), Type::Float);
}
