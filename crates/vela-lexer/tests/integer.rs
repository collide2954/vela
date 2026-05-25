use vela_lexer::{Token, TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
    let mut ks: Vec<TokenKind> = lex(src).map(|t| t.kind).collect();
    while matches!(ks.last(), Some(TokenKind::Newline | TokenKind::Dedent)) {
        ks.pop();
    }
    ks
}

#[test]
fn single_zero() {
    assert_eq!(kinds("0"), vec![TokenKind::Int(0)]);
}

#[test]
fn single_positive_integer() {
    assert_eq!(kinds("42"), vec![TokenKind::Int(42)]);
}

#[test]
fn integer_with_digit_separators() {
    assert_eq!(kinds("1_000_000"), vec![TokenKind::Int(1_000_000)]);
}

#[test]
fn integer_span_covers_full_literal() {
    let toks: Vec<Token> = lex("42 ").collect();
    let int_tok = toks
        .iter()
        .find(|t| matches!(t.kind, TokenKind::Int(_)))
        .expect("found an Int token");
    assert_eq!(int_tok.span, 0..2);
}
