use vela_lexer::{TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
    lex(src).map(|t| t.kind).collect()
}

#[test]
fn empty_string() {
    assert_eq!(kinds(r#""""#), vec![TokenKind::Str(String::new())]);
}

#[test]
fn simple_string() {
    assert_eq!(kinds(r#""hello""#), vec![TokenKind::Str("hello".into())]);
}

#[test]
fn string_with_newline_escape() {
    assert_eq!(kinds(r#""line\nbreak""#), vec![TokenKind::Str("line\nbreak".into())]);
}

#[test]
fn string_with_tab_escape() {
    assert_eq!(kinds(r#""a\tb""#), vec![TokenKind::Str("a\tb".into())]);
}

#[test]
fn string_with_carriage_return_escape() {
    assert_eq!(kinds(r#""a\rb""#), vec![TokenKind::Str("a\rb".into())]);
}

#[test]
fn string_with_quote_escape() {
    assert_eq!(kinds(r#""a\"b""#), vec![TokenKind::Str("a\"b".into())]);
}

#[test]
fn string_with_backslash_escape() {
    assert_eq!(kinds(r#""a\\b""#), vec![TokenKind::Str("a\\b".into())]);
}

#[test]
fn string_with_null_escape() {
    assert_eq!(kinds(r#""a\0b""#), vec![TokenKind::Str("a\0b".into())]);
}

#[test]
fn string_span_includes_quotes() {
    let toks: Vec<_> = lex(r#""hi""#).collect();
    assert_eq!(toks.len(), 1);
    assert_eq!(toks[0].span, 0..4);
}
