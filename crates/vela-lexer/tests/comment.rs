use vela_lexer::{TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
    lex(src).map(|t| t.kind).collect()
}

#[test]
fn hash_comment_skipped() {
    assert_eq!(kinds("# nothing to see\n42"), vec![TokenKind::Int(42)]);
}

#[test]
fn hash_comment_at_eof_skipped() {
    assert_eq!(kinds("42 # trailing"), vec![TokenKind::Int(42)]);
}

#[test]
fn slash_slash_is_skipped_too() {
    assert_eq!(kinds("// also a comment\n7"), vec![TokenKind::Int(7)]);
}

#[test]
fn doc_comment_triple_slash() {
    assert_eq!(
        kinds("/// the answer\n"),
        vec![TokenKind::DocComment("the answer".into())],
    );
}

#[test]
fn mod_doc_comment() {
    assert_eq!(
        kinds("//! module top\n"),
        vec![TokenKind::ModDoc("module top".into())],
    );
}

#[test]
fn doc_comment_then_let() {
    assert_eq!(
        kinds("/// doc\nlet"),
        vec![
            TokenKind::DocComment("doc".into()),
            TokenKind::Keyword(vela_lexer::Keyword::Let),
        ],
    );
}

#[test]
fn slash_alone_is_division_operator() {
    assert_eq!(
        kinds("a / b"),
        vec![
            TokenKind::Ident("a".into()),
            TokenKind::Op(vela_lexer::Op::Slash),
            TokenKind::Ident("b".into()),
        ],
    );
}

#[test]
fn empty_doc_comment() {
    assert_eq!(kinds("///\n"), vec![TokenKind::DocComment(String::new())]);
}
