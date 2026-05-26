use vela_eval::{Value, run};

fn r(src: &str) -> Value {
    run(src).expect("runs")
}

#[test]
fn map_increments_series() {
    let src = "map (fn x -> x + 1) [1, 2, 3]";
    assert_eq!(
        r(src),
        Value::Series(vec![Value::Int(2), Value::Int(3), Value::Int(4)]),
    );
}

#[test]
fn filter_keeps_predicate_true() {
    let src = "filter (fn x -> x > 2) [1, 2, 3, 4]";
    assert_eq!(r(src), Value::Series(vec![Value::Int(3), Value::Int(4)]),);
}

#[test]
fn fold_sums_series() {
    let src = "fold (fn acc x -> acc + x) 0 [1, 2, 3, 4]";
    assert_eq!(r(src), Value::Int(10));
}

#[test]
fn length_of_series_and_string() {
    assert_eq!(r("length [1, 2, 3]"), Value::Int(3));
    assert_eq!(r(r#"length "abc""#), Value::Int(3));
}

#[test]
fn sum_int_and_float() {
    assert_eq!(r("sum [1, 2, 3, 4]"), Value::Int(10));
    assert_eq!(r("sum [1.0, 2.0, 3.0]"), Value::Float(6.0));
}

#[test]
fn mean_of_float_series() {
    assert_eq!(r("mean [1.0, 2.0, 3.0, 4.0]"), Value::Float(2.5));
}

#[test]
fn min_max_of_floats() {
    assert_eq!(r("min [3.0, 1.0, 2.0]"), Value::Float(1.0));
    assert_eq!(r("max [3.0, 1.0, 2.0]"), Value::Float(3.0));
}

#[test]
fn float_of_int_conversion() {
    assert_eq!(r("Float.of_int 7"), Value::Float(7.0));
}

#[test]
fn option_unwrap_some() {
    assert_eq!(r("Option.unwrap (Some 42)"), Value::Int(42));
}

#[test]
fn result_unwrap_ok() {
    assert_eq!(r("Result.unwrap (Ok 42)"), Value::Int(42));
}

#[test]
fn string_namespace_concat() {
    assert_eq!(
        r(r#"String.concat ["a", "b", "c"]"#),
        Value::Str("abc".into()),
    );
}

#[test]
fn standardize_with_map_mean() {
    let src = "let xs = [1.0, 2.0, 3.0]\nlet m = mean xs\nmap (fn x -> x - m) xs";
    assert_eq!(
        r(src),
        Value::Series(vec![
            Value::Float(-1.0),
            Value::Float(0.0),
            Value::Float(1.0)
        ]),
    );
}
