use vela_lexer::{TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
    let mut ks: Vec<TokenKind> = lex(src).map(|t| t.kind).collect();
    while matches!(ks.last(), Some(TokenKind::Newline | TokenKind::Dedent)) {
        ks.pop();
    }
    ks
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
    assert_eq!(
        kinds(r#""line\nbreak""#),
        vec![TokenKind::Str("line\nbreak".into())]
    );
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
    let s = toks
        .iter()
        .find(|t| matches!(t.kind, TokenKind::Str(_)))
        .expect("found a Str token");
    assert_eq!(s.span, 0..4);
}
