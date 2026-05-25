use vela_diag::{Diagnostic, Severity};

#[test]
fn error_without_span() {
    let d = Diagnostic::error("oops");
    let out = d.render("");
    assert!(out.starts_with("error: oops"));
}

#[test]
fn error_with_code() {
    let d = Diagnostic::error("type mismatch").with_code("E0123");
    let out = d.render("");
    assert!(out.starts_with("error[E0123]: type mismatch"));
}

#[test]
fn error_with_span_shows_line_and_caret() {
    let src = "let x = 1\nlet y = \"abc\"\nlet z = 2";
    // Span of the string literal "abc" on line 2.
    let start = src.find('"').unwrap();
    let end = start + 5;
    let d = Diagnostic::error("expected Int, found String")
        .with_code("E0001")
        .with_span(start..end)
        .with_path("example.vela");
    let out = d.render(src);
    assert!(out.contains("error[E0001]: expected Int, found String"));
    assert!(out.contains(" --> example.vela:2:9"));
    assert!(out.contains("let y = \"abc\""));
    assert!(out.contains("^^^^^"));
}

#[test]
fn warning_severity() {
    let d = Diagnostic {
        severity: Severity::Warning,
        code: Some("W0042".into()),
        message: "unused binding".into(),
        primary: None,
        source_path: "x.vela".into(),
    };
    let out = d.render("");
    assert!(out.starts_with("warning[W0042]: unused binding"));
}
