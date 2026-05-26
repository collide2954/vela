use vela_eval::{Value, run};

fn r(src: &str) -> Value {
    run(src).expect("runs")
}

#[test]
fn head_of_nonempty() {
    assert_eq!(
        r("head [1, 2, 3]"),
        Value::Cons("Some".into(), vec![Value::Int(1)])
    );
}

#[test]
fn head_of_empty() {
    assert_eq!(r("head []"), Value::Cons("None".into(), vec![]));
}

#[test]
fn tail_skips_first() {
    assert_eq!(
        r("tail [1, 2, 3]"),
        Value::Series(vec![Value::Int(2), Value::Int(3)]),
    );
}

#[test]
fn take_first_n() {
    assert_eq!(
        r("take 2 [1, 2, 3, 4]"),
        Value::Series(vec![Value::Int(1), Value::Int(2)]),
    );
}

#[test]
fn drop_first_n() {
    assert_eq!(
        r("drop 2 [1, 2, 3, 4]"),
        Value::Series(vec![Value::Int(3), Value::Int(4)]),
    );
}

#[test]
fn reverse_series() {
    assert_eq!(
        r("reverse [1, 2, 3]"),
        Value::Series(vec![Value::Int(3), Value::Int(2), Value::Int(1)]),
    );
}

#[test]
fn append_two_series() {
    assert_eq!(
        r("append [1, 2] [3, 4]"),
        Value::Series(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4)
        ]),
    );
}

#[test]
fn zip_pairs_elements() {
    assert_eq!(
        r("zip [1, 2, 3] [10, 20, 30]"),
        Value::Series(vec![
            Value::Tuple(vec![Value::Int(1), Value::Int(10)]),
            Value::Tuple(vec![Value::Int(2), Value::Int(20)]),
            Value::Tuple(vec![Value::Int(3), Value::Int(30)]),
        ]),
    );
}

#[test]
fn enumerate_indices_elements() {
    assert_eq!(
        r(r#"enumerate ["a", "b"]"#),
        Value::Series(vec![
            Value::Tuple(vec![Value::Int(0), Value::Str("a".into())]),
            Value::Tuple(vec![Value::Int(1), Value::Str("b".into())]),
        ]),
    );
}

#[test]
fn range_half_open() {
    assert_eq!(
        r("range 0 4"),
        Value::Series(vec![
            Value::Int(0),
            Value::Int(1),
            Value::Int(2),
            Value::Int(3)
        ]),
    );
}

#[test]
fn pipeline_with_new_builtins() {
    let src = "range 1 11 |> filter (fn n -> n > 5) |> sum";
    assert_eq!(r(src), Value::Int(40));
}
