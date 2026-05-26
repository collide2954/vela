use vela_eval::{Value, run};

fn r(src: &str) -> Value {
    run(src).expect("runs")
}

#[test]
fn format_with_two_args() {
    let src = r#"format "{} = {}" "name" 42"#;
    assert_eq!(r(src), Value::Str("name = 42".into()));
}

#[test]
fn format_no_holes_returns_template() {
    let src = r#"format "hello""#;
    assert_eq!(r(src), Value::Str("hello".into()));
}

#[test]
fn format_three_args_mixed_types() {
    let src = r#"format "x={}, y={}, ok={}" 1 2.5 true"#;
    assert_eq!(r(src), Value::Str("x=1, y=2.5, ok=true".into()));
}

#[test]
fn show_int() {
    assert_eq!(r("show 42"), Value::Str("42".into()));
}

#[test]
fn show_tuple() {
    assert_eq!(r("show (1, true)"), Value::Str("(1, true)".into()));
}
