use vela_eval::{Value, run};

fn r(src: &str) -> Value {
    run(src).expect("runs")
}

#[test]
fn question_unwraps_ok() {
    let src = "let chain x =\n    let a = x?\n    Ok (a + 1)\nchain (Ok 10)";
    assert_eq!(r(src), Value::Cons("Ok".into(), vec![Value::Int(11)]));
}

#[test]
fn question_short_circuits_err() {
    let src = "let chain x =\n    let a = x?\n    Ok (a + 1)\nchain (Err \"bad\")";
    assert_eq!(
        r(src),
        Value::Cons("Err".into(), vec![Value::Str("bad".into())]),
    );
}

#[test]
fn question_threads_through_chain() {
    let src =
        "let chain a b =\n    let x = a?\n    let y = b?\n    Ok (x + y)\nchain (Ok 3) (Ok 4)";
    assert_eq!(r(src), Value::Cons("Ok".into(), vec![Value::Int(7)]));
}

#[test]
fn question_short_circuits_second_step() {
    let src = "let chain a b =\n    let x = a?\n    let y = b?\n    Ok (x + y)\nchain (Ok 3) (Err \"oops\")";
    assert_eq!(
        r(src),
        Value::Cons("Err".into(), vec![Value::Str("oops".into())]),
    );
}
