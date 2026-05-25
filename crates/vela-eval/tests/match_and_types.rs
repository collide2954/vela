use vela_eval::{Value, run};

fn r(src: &str) -> Value {
    run(src).expect("runs")
}

#[test]
fn match_on_int_literal() {
    let src = r#"match 1 with
| 0 -> "zero"
| 1 -> "one"
| _ -> "other""#;
    assert_eq!(r(src), Value::Str("one".into()));
}

#[test]
fn match_with_var_binding() {
    let src = r#"match 5 with
| n -> n * 2"#;
    assert_eq!(r(src), Value::Int(10));
}

#[test]
fn match_some_extracts_value() {
    let src = r#"match Some 7 with
| None -> 0
| Some x -> x"#;
    assert_eq!(r(src), Value::Int(7));
}

#[test]
fn match_none_takes_default() {
    let src = r#"match None with
| None -> 42
| Some x -> x"#;
    assert_eq!(r(src), Value::Int(42));
}

#[test]
fn user_sum_constructor_and_match() {
    let src = r#"type Shape =
    | Circle Float
    | Square Float

let area s =
    match s with
    | Circle r -> r * r
    | Square s -> s * s

area (Circle 3.0)"#;
    assert_eq!(r(src), Value::Float(9.0));
}

#[test]
fn match_or_pattern() {
    let src = r#"type Color = | Red | Blue | Green

match Blue with
| Red | Blue -> "warm"
| Green -> "cool""#;
    assert_eq!(r(src), Value::Str("warm".into()));
}

#[test]
fn match_with_guard() {
    let src = r#"match 5 with
| n when n > 0 -> "positive"
| _ -> "nonpositive""#;
    assert_eq!(r(src), Value::Str("positive".into()));
}

#[test]
fn tuple_destructuring_in_match() {
    let src = r#"match (1, 2) with
| (a, b) -> a + b"#;
    assert_eq!(r(src), Value::Int(3));
}

#[test]
fn for_loop_with_var_accumulator() {
    let src = r#"var total = 0
for x in [1, 2, 3, 4]:
    total <- total + x
total"#;
    assert_eq!(r(src), Value::Int(10));
}

#[test]
fn destructuring_let() {
    let src = r#"let (a, b) = (10, 20)
a + b"#;
    assert_eq!(r(src), Value::Int(30));
}

#[test]
fn record_destructuring() {
    let src = r#"let { x, y } = { x = 3, y = 4 }
x + y"#;
    assert_eq!(r(src), Value::Int(7));
}

#[test]
fn list_head_tail_pattern() {
    let src = r#"match [1, 2, 3, 4] with
| [head, ..rest] -> head
| _ -> 0"#;
    assert_eq!(r(src), Value::Int(1));
}
