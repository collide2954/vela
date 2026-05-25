use vela_lexer::{TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
    let mut ks: Vec<TokenKind> = lex(src).map(|t| t.kind).collect();
    while matches!(ks.last(), Some(TokenKind::Newline | TokenKind::Dedent)) {
        ks.pop();
    }
    ks
}

#[test]
fn simple_float() {
    assert_eq!(kinds("2.5"), vec![TokenKind::Float(2.5)]);
}

#[test]
fn float_with_separators() {
    assert_eq!(kinds("1_000.500_5"), vec![TokenKind::Float(1000.5005)]);
}

#[test]
fn float_scientific_lower() {
    assert_eq!(kinds("1e-3"), vec![TokenKind::Float(1e-3)]);
}

#[test]
fn float_scientific_upper_with_plus() {
    assert_eq!(kinds("2.5E+10"), vec![TokenKind::Float(2.5e10)]);
}

#[test]
fn integer_with_exponent_is_float() {
    assert_eq!(kinds("4e2"), vec![TokenKind::Float(400.0)]);
}

#[test]
fn float_nan_constant() {
    let toks = kinds("NaN");
    assert_eq!(toks.len(), 1);
    match &toks[0] {
        TokenKind::Float(f) => assert!(f.is_nan()),
        other => panic!("expected float, got {other:?}"),
    }
}

#[test]
fn float_inf_constant() {
    assert_eq!(kinds("Inf"), vec![TokenKind::Float(f64::INFINITY)]);
}
