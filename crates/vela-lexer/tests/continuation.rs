use vela_lexer::{Op, TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
    let mut ks: Vec<TokenKind> = lex(src).map(|t| t.kind).collect();
    while matches!(ks.last(), Some(TokenKind::Newline | TokenKind::Dedent)) {
        ks.pop();
    }
    ks
}

#[test]
fn pipe_at_start_of_next_line_suppresses_newline() {
    assert_eq!(
        kinds("df\n|> f"),
        vec![
            TokenKind::Ident("df".into()),
            TokenKind::Op(Op::Pipe),
            TokenKind::Ident("f".into()),
        ],
    );
}

#[test]
fn double_plus_at_start_of_next_line_suppresses_newline() {
    assert_eq!(
        kinds("a\n++ b"),
        vec![
            TokenKind::Ident("a".into()),
            TokenKind::Op(Op::PlusPlus),
            TokenKind::Ident("b".into()),
        ],
    );
}

#[test]
fn three_line_pipeline_is_one_logical_line() {
    let toks = kinds("df\n|> filter x\n|> map g");
    assert_eq!(
        toks,
        vec![
            TokenKind::Ident("df".into()),
            TokenKind::Op(Op::Pipe),
            TokenKind::Ident("filter".into()),
            TokenKind::Ident("x".into()),
            TokenKind::Op(Op::Pipe),
            TokenKind::Ident("map".into()),
            TokenKind::Ident("g".into()),
        ],
    );
}

#[test]
fn non_continuation_op_keeps_newline() {
    // `+` at line start is not a continuation marker (could be a prefix
    // operator); newline is preserved.
    let toks = kinds("a\n+ b");
    assert!(toks.contains(&TokenKind::Newline));
}
