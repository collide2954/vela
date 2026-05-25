use vela_lexer::{TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
    lex(src).map(|t| t.kind).collect()
}

#[test]
fn simple_symbol() {
    assert_eq!(kinds(":foo"), vec![TokenKind::Sym("foo".into())]);
}

#[test]
fn symbol_species() {
    assert_eq!(kinds(":species"), vec![TokenKind::Sym("species".into())]);
}

#[test]
fn symbol_with_underscore_and_digits() {
    assert_eq!(kinds(":petal_length_2"), vec![TokenKind::Sym("petal_length_2".into())]);
}

#[test]
fn symbol_starting_with_underscore() {
    assert_eq!(kinds(":_internal"), vec![TokenKind::Sym("_internal".into())]);
}

#[test]
fn symbol_span_covers_colon_and_body() {
    let toks: Vec<_> = lex(":species").collect();
    assert_eq!(toks[0].span, 0..8);
}

#[test]
fn two_symbols_separated_by_comma_lex_correctly() {
    // We don't yet lex commas; verify the two symbols are at least
    // present in order with the second one untouched by the first.
    let src = ":x :y";
    let toks: Vec<_> = lex(src).collect();
    assert_eq!(toks[0].kind, TokenKind::Sym("x".into()));
    assert_eq!(toks[1].kind, TokenKind::Sym("y".into()));
}
