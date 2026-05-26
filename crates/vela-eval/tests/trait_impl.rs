use vela_eval::{Value, run};

fn r(src: &str) -> Value {
    run(src).expect("runs")
}

#[test]
fn impl_method_callable() {
    let src = "trait Show t =\n    fn show (x : t) : String\n\nimpl Show Int =\n    fn show x = Int.to_string x\n\nshow 42";
    assert_eq!(r(src), Value::Str("42".into()));
}

#[test]
fn impl_method_can_close_over_helpers() {
    let src = "let double n = n * 2\ntrait Double t =\n    fn dbl (x : t) : t\n\nimpl Double Int =\n    fn dbl x = double x\n\ndbl 21";
    assert_eq!(r(src), Value::Int(42));
}
