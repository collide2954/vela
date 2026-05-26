use vela_eval::{Value, run};

fn r(src: &str) -> Value {
    run(src).expect("runs")
}

#[test]
fn literal_int() {
    assert_eq!(r("42"), Value::Int(42));
}

#[test]
fn arithmetic() {
    assert_eq!(r("1 + 2 * 3"), Value::Int(7));
    assert_eq!(r("(1 + 2) * 3"), Value::Int(9));
    assert_eq!(r("10 - 4"), Value::Int(6));
    assert_eq!(r("2 ^ 10"), Value::Int(1024));
}

#[test]
fn float_arithmetic() {
    assert_eq!(r("1.5 + 2.5"), Value::Float(4.0));
    assert_eq!(r("1.0 / 2.0"), Value::Float(0.5));
}

#[test]
fn let_binding_and_use() {
    assert_eq!(r("let x = 5\nlet y = 10\nx + y"), Value::Int(15));
}

#[test]
fn lambda_application() {
    assert_eq!(r("(fn x -> x + 1) 5"), Value::Int(6));
}

#[test]
fn curried_function() {
    let src = "let add = fn x y -> x + y\nadd 3 4";
    assert_eq!(r(src), Value::Int(7));
}

#[test]
fn function_definition_sugar() {
    let src = "let mul x y = x * y\nmul 6 7";
    assert_eq!(r(src), Value::Int(42));
}

#[test]
fn if_then_else() {
    assert_eq!(r("if true then 1 else 0"), Value::Int(1));
    assert_eq!(r("if false then 1 else 0"), Value::Int(0));
}

#[test]
fn comparisons() {
    assert_eq!(r("1 < 2"), Value::Bool(true));
    assert_eq!(r("1 == 1"), Value::Bool(true));
    assert_eq!(r("1 != 2"), Value::Bool(true));
}

#[test]
fn string_concat() {
    assert_eq!(r(r#""hello " ++ "world""#), Value::Str("hello world".into()));
}

#[test]
fn series_concat() {
    assert_eq!(
        r("[1, 2] ++ [3, 4]"),
        Value::Series(vec![
            Value::Int(1),
            Value::Int(2),
            Value::Int(3),
            Value::Int(4),
        ]),
    );
}

#[test]
fn tuple_literal() {
    assert_eq!(
        r(r#"(1, "x", true)"#),
        Value::Tuple(vec![
            Value::Int(1),
            Value::Str("x".into()),
            Value::Bool(true),
        ]),
    );
}

#[test]
fn record_field_access() {
    assert_eq!(r("{ x = 1, y = 2 }.x"), Value::Int(1));
    assert_eq!(r("{ x = 1, y = 2 }.y"), Value::Int(2));
}

#[test]
fn let_rec_factorial() {
    let src = "let rec fact n = if n == 0 then 1 else n * fact (n - 1)\nfact 5";
    assert_eq!(r(src), Value::Int(120));
}

#[test]
fn let_rec_fibonacci() {
    let src =
        "let rec fib n = if n < 2 then n else fib (n - 1) + fib (n - 2)\nfib 10";
    assert_eq!(r(src), Value::Int(55));
}

#[test]
fn let_rec_mutual_even_odd() {
    let src = "let rec is_even n = if n == 0 then true else is_odd (n - 1)\nand is_odd n = if n == 0 then false else is_even (n - 1)\nis_even 10";
    assert_eq!(r(src), Value::Bool(true));
}

#[test]
fn non_recursive_let_shadows_outer() {
    let src = "let x = 1\nlet x = x + 1\nx";
    assert_eq!(r(src), Value::Int(2));
}

#[test]
fn pipeline() {
    let src = "let inc x = x + 1\n5 |> inc";
    assert_eq!(r(src), Value::Int(6));
}

#[test]
fn nested_lambdas() {
    let src = r#"let make_adder = fn x -> fn y -> x + y
let add5 = make_adder 5
add5 10"#;
    assert_eq!(r(src), Value::Int(15));
}

#[test]
fn unary_neg() {
    assert_eq!(r("-(1 + 2)"), Value::Int(-3));
}

#[test]
fn boolean_and_or_not() {
    assert_eq!(r("true and false"), Value::Bool(false));
    assert_eq!(r("true or false"), Value::Bool(true));
    assert_eq!(r("not true"), Value::Bool(false));
}

#[test]
fn lambda_unit_param() {
    let src = "let thunk = fn () -> 42\nthunk ()";
    assert_eq!(r(src), Value::Int(42));
}

#[test]
fn lambda_tuple_param() {
    let src = "let fst = fn (a, b) -> a\nfst (3, 4)";
    assert_eq!(r(src), Value::Int(3));
}

#[test]
fn lambda_block_body() {
    let src = "let f = fn x ->\n    let y = x + 1\n    y * 2\nf 3";
    assert_eq!(r(src), Value::Int(8));
}

#[test]
fn match_arm_block_body() {
    let src = "let v = Some 3\nmatch v with\n| Some x ->\n    let y = x + 1\n    y * 2\n| None -> 0";
    assert_eq!(r(src), Value::Int(8));
}
