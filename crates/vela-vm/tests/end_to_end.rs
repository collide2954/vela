use vela_compile::compile_source;
use vela_vm::{Value, run};

fn r(src: &str) -> Value {
    let module = compile_source(src).expect("compiles");
    run(&module).expect("runs")
}

#[test]
fn int_literal() {
    assert_eq!(r("42"), Value::Int(42));
}

#[test]
fn arithmetic_ints() {
    assert_eq!(r("1 + 2 * 3"), Value::Int(7));
    assert_eq!(r("(1 + 2) * 3"), Value::Int(9));
    assert_eq!(r("10 - 4"), Value::Int(6));
    assert_eq!(r("2 ^ 8"), Value::Int(256));
}

#[test]
fn arithmetic_floats() {
    assert_eq!(r("1.5 + 2.5"), Value::Float(4.0));
    assert_eq!(r("1.0 / 2.0"), Value::Float(0.5));
}

#[test]
fn comparisons() {
    assert_eq!(r("1 < 2"), Value::Bool(true));
    assert_eq!(r("3 == 3"), Value::Bool(true));
    assert_eq!(r("5 != 5"), Value::Bool(false));
}

#[test]
fn let_then_use() {
    assert_eq!(r("let x = 5\nlet y = 10\nx + y"), Value::Int(15));
}

#[test]
fn let_shadowing_picks_latest() {
    assert_eq!(r("let x = 1\nlet x = x + 1\nx"), Value::Int(2));
}

#[test]
fn if_then_else_picks_branch() {
    assert_eq!(r("if true then 1 else 0"), Value::Int(1));
    assert_eq!(r("if false then 1 else 0"), Value::Int(0));
}

#[test]
fn unary_neg_and_not() {
    assert_eq!(r("-(1 + 2)"), Value::Int(-3));
    assert_eq!(r("not (1 == 2)"), Value::Bool(true));
}

#[test]
fn string_concat() {
    assert_eq!(r(r#""ab" ++ "cd""#), Value::Str("abcd".into()));
}
