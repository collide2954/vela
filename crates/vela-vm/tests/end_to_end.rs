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

#[test]
fn lambda_application() {
    assert_eq!(r("(fn x -> x + 1) 5"), Value::Int(6));
}

#[test]
fn named_function_one_arg() {
    assert_eq!(r("let inc x = x + 1\ninc 41"), Value::Int(42));
}

#[test]
fn curried_function_two_args() {
    assert_eq!(r("let add x y = x + y\nadd 3 4"), Value::Int(7));
}

#[test]
fn closure_captures_outer_local() {
    assert_eq!(
        r("let n = 10\nlet add_n x = x + n\nadd_n 5"),
        Value::Int(15),
    );
}

#[test]
fn nested_lambda_capture_chain() {
    assert_eq!(
        r("let make_adder x = fn y -> x + y\nlet add5 = make_adder 5\nadd5 10"),
        Value::Int(15),
    );
}

#[test]
fn tuple_literal() {
    use std::rc::Rc;
    assert_eq!(
        r("(1, 2, 3)"),
        Value::Tuple(Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)])),
    );
}

#[test]
fn series_literal() {
    use std::rc::Rc;
    assert_eq!(
        r("[1, 2, 3]"),
        Value::Series(Rc::new(vec![Value::Int(1), Value::Int(2), Value::Int(3)])),
    );
}

#[test]
fn record_literal_and_field_access() {
    let src = "let p = { x = 1, y = 2 }\np.x + p.y";
    assert_eq!(r(src), Value::Int(3));
}

#[test]
fn record_then_function() {
    let src = "let project p = p.x\nproject { x = 7, y = 8 }";
    assert_eq!(r(src), Value::Int(7));
}

#[test]
fn none_constructor() {
    use std::rc::Rc;
    use vela_vm::ConsValue;
    assert_eq!(
        r("None"),
        Value::Cons(Rc::new(ConsValue {
            name: "None".into(),
            args: vec![],
        })),
    );
}

#[test]
fn some_constructor_applied() {
    use std::rc::Rc;
    use vela_vm::ConsValue;
    assert_eq!(
        r("Some 42"),
        Value::Cons(Rc::new(ConsValue {
            name: "Some".into(),
            args: vec![Value::Int(42)],
        })),
    );
}

#[test]
fn ok_constructor_through_let() {
    use std::rc::Rc;
    use vela_vm::ConsValue;
    let src = "let x = Ok 7\nx";
    assert_eq!(
        r(src),
        Value::Cons(Rc::new(ConsValue {
            name: "Ok".into(),
            args: vec![Value::Int(7)],
        })),
    );
}

#[test]
fn match_literal_int() {
    let src = "match 2 with\n| 1 -> 10\n| 2 -> 20\n| _ -> 0";
    assert_eq!(r(src), Value::Int(20));
}

#[test]
fn match_option_some_extracts() {
    let src = "match Some 7 with\n| None -> 0\n| Some x -> x";
    assert_eq!(r(src), Value::Int(7));
}

#[test]
fn match_option_none() {
    let src = "let x = None\nmatch x with\n| None -> -1\n| Some _ -> 0";
    assert_eq!(r(src), Value::Int(-1));
}

#[test]
fn match_result_chained() {
    let src = "let f r = match r with\n| Ok n -> n + 1\n| Err _ -> 0\nf (Ok 41)";
    assert_eq!(r(src), Value::Int(42));
}

#[test]
fn match_nested_constructor() {
    let src = "let v = Some (Ok 100)\nmatch v with\n| None -> 0\n| Some (Err _) -> -1\n| Some (Ok n) -> n";
    assert_eq!(r(src), Value::Int(100));
}

#[test]
fn match_var_binds() {
    let src = "match 42 with\n| n -> n + 1";
    assert_eq!(r(src), Value::Int(43));
}

#[test]
fn constructor_passed_to_function() {
    use std::rc::Rc;
    use vela_vm::ConsValue;
    let src = "let id x = x\nid (Err \"bad\")";
    assert_eq!(
        r(src),
        Value::Cons(Rc::new(ConsValue {
            name: "Err".into(),
            args: vec![Value::Str("bad".into())],
        })),
    );
}
