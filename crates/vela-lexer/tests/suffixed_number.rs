use vela_lexer::{TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
    lex(src).map(|t| t.kind).collect()
}

#[test]
fn uint_zero() {
    assert_eq!(kinds("0u"), vec![TokenKind::UInt(0)]);
}

#[test]
fn uint_value() {
    assert_eq!(kinds("42u"), vec![TokenKind::UInt(42)]);
}

#[test]
fn uint_with_separators() {
    assert_eq!(kinds("1_000u"), vec![TokenKind::UInt(1000)]);
}

#[test]
fn bigint_zero() {
    assert_eq!(kinds("0n"), vec![TokenKind::BigInt("0".into())]);
}

#[test]
fn bigint_value() {
    assert_eq!(kinds("42n"), vec![TokenKind::BigInt("42".into())]);
}

#[test]
fn bigint_with_separators() {
    assert_eq!(kinds("1_000n"), vec![TokenKind::BigInt("1000".into())]);
}

#[test]
fn decimal_with_dot() {
    assert_eq!(kinds("1.50d"), vec![TokenKind::Decimal("1.50".into())]);
}

#[test]
fn decimal_whole() {
    assert_eq!(kinds("42d"), vec![TokenKind::Decimal("42".into())]);
}

#[test]
fn suffix_requires_word_boundary() {
    // `42un` should not be UInt(42) Ident(n); the `un` is two letters,
    // so the suffix `u` is invalid here. We treat the digits as Int and
    // leave the lexer at the first letter to be handled separately.
    let toks = kinds("42 un");
    assert_eq!(toks[0], TokenKind::Int(42));
}

#[test]
fn digits_then_space_then_letter() {
    let toks = kinds("42 n");
    assert_eq!(toks[0], TokenKind::Int(42));
}
