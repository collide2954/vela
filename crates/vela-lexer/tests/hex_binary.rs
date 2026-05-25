use vela_lexer::{TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
    lex(src).map(|t| t.kind).collect()
}

#[test]
fn hex_lowercase() {
    assert_eq!(kinds("0xff"), vec![TokenKind::Int(0xff)]);
}

#[test]
fn hex_uppercase() {
    assert_eq!(kinds("0xFF"), vec![TokenKind::Int(0xff)]);
}

#[test]
fn hex_mixed_case_with_separators() {
    assert_eq!(kinds("0xDEAD_beef"), vec![TokenKind::Int(0xdead_beef)]);
}

#[test]
fn hex_zero() {
    assert_eq!(kinds("0x0"), vec![TokenKind::Int(0)]);
}

#[test]
fn binary_simple() {
    assert_eq!(kinds("0b10"), vec![TokenKind::Int(0b10)]);
}

#[test]
fn binary_with_separators() {
    assert_eq!(kinds("0b1010_1010"), vec![TokenKind::Int(0b1010_1010)]);
}

#[test]
fn binary_zero() {
    assert_eq!(kinds("0b0"), vec![TokenKind::Int(0)]);
}

#[test]
fn hex_uint_suffix() {
    assert_eq!(kinds("0xffu"), vec![TokenKind::UInt(0xff)]);
}
