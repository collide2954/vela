use vela_check::check_program_with_warnings;

#[test]
fn shadowing_let_emits_w0050() {
    let src = "let x = 1\nlet x = x + 1\nx";
    let (_, ws) = check_program_with_warnings(src).expect("type-checks");
    assert!(
        ws.iter()
            .any(|w| w.code == "W0050" && w.message.contains("x"))
    );
}

#[test]
fn no_warning_when_no_shadowing() {
    let src = "let x = 1\nlet y = 2\nx + y";
    let (_, ws) = check_program_with_warnings(src).expect("type-checks");
    assert!(ws.is_empty());
}

#[test]
fn var_shadowing_also_warns() {
    let src = "let n = 1\nvar n = 2\nn";
    let (_, ws) = check_program_with_warnings(src).expect("type-checks");
    assert!(ws.iter().any(|w| w.code == "W0050"));
}
