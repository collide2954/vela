use vela_fmt::format_source;
use vela_parser::parse_program;

fn fmt(src: &str) -> String {
    format_source(src).expect("formats")
}

fn roundtrip_stable(src: &str) {
    let once = fmt(src);
    let twice = fmt(&once);
    assert_eq!(once, twice, "format is not idempotent");
    let original_ast = parse_program(src).expect("orig parses");
    let formatted_ast = parse_program(&once).expect("formatted parses");
    assert_eq!(
        original_ast, formatted_ast,
        "format changed program meaning",
    );
}

#[test]
fn let_binding() {
    roundtrip_stable("let x = 1");
}

#[test]
fn let_with_function() {
    roundtrip_stable("let add x y = x + y");
}

#[test]
fn let_with_annotated_params() {
    roundtrip_stable("let standardize (xs : [Float]) : [Float] = xs");
}

#[test]
fn let_rec() {
    roundtrip_stable("let rec fact n = if n == 0 then 1 else n * fact (n - 1)");
}

#[test]
fn let_rec_mutual() {
    let src = "let rec is_even n =\n    if n == 0 then true else is_odd (n - 1)\nand is_odd n =\n    if n == 0 then false else is_even (n - 1)";
    roundtrip_stable(src);
}

#[test]
fn type_sum() {
    let src = "type Shape =\n    | Circle Float\n    | Square Float\n    | Rect { width : Float, height : Float }";
    roundtrip_stable(src);
}

#[test]
fn nominal_record_alias() {
    roundtrip_stable("type Point = { x : Float, y : Float }");
}

#[test]
fn match_expression() {
    let src = "let area shape =\n    match shape with\n    | Circle r -> 3.14 * r * r\n    | Square s -> s * s";
    roundtrip_stable(src);
}

#[test]
fn pipeline() {
    roundtrip_stable("xs |> map (fn x -> x + 1) |> filter (fn x -> x > 0)");
}

#[test]
fn import_grouping() {
    let src = "import std.io\nimport vendor.foo\nimport std.stats";
    let formatted = fmt(src);
    assert!(formatted.contains("import std.io\nimport std.stats"));
    assert!(formatted.contains("import vendor.foo"));
}

#[test]
fn records_inline_and_block() {
    roundtrip_stable("let p = { x = 1.0, y = 2.0 }");
}

#[test]
fn dataframe_literal() {
    roundtrip_stable("let df = {| x : [1, 2], y : [3, 4] |}");
}

#[test]
fn for_loop_with_pattern() {
    let src = "for (a, b) in pairs:\n    println a";
    roundtrip_stable(src);
}

#[test]
fn trait_and_impl() {
    let src = "trait Show t =\n    fn show (x : t) : String\n\nimpl Show Int =\n    fn show x = Int.to_string x";
    roundtrip_stable(src);
}

#[test]
fn realistic_program() {
    let src = "import std.stats\n\nlet standardize xs =\n    let m = mean xs\n    let s = std xs\n    xs |> map (fn x -> (x - m) / s)\n\ntype Shape =\n    | Circle Float\n    | Square Float\n\nlet area shape =\n    match shape with\n    | Circle r -> 3.14 * r * r\n    | Square s -> s * s";
    roundtrip_stable(src);
}
