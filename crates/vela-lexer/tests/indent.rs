use vela_lexer::{Keyword, Op, TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
    lex(src).map(|t| t.kind).collect()
}

fn k(k: Keyword) -> TokenKind {
    TokenKind::Keyword(k)
}
fn id(s: &str) -> TokenKind {
    TokenKind::Ident(s.into())
}

#[test]
fn single_line_ends_with_newline_at_eof() {
    assert_eq!(
        kinds("let x = 1"),
        vec![
            k(Keyword::Let),
            id("x"),
            TokenKind::Op(Op::Assign),
            TokenKind::Int(1),
            TokenKind::Newline,
        ],
    );
}

#[test]
fn two_top_level_statements() {
    assert_eq!(
        kinds("let x = 1\nlet y = 2\n"),
        vec![
            k(Keyword::Let),
            id("x"),
            TokenKind::Op(Op::Assign),
            TokenKind::Int(1),
            TokenKind::Newline,
            k(Keyword::Let),
            id("y"),
            TokenKind::Op(Op::Assign),
            TokenKind::Int(2),
            TokenKind::Newline,
        ],
    );
}

#[test]
fn indented_block_emits_indent_and_dedent() {
    let src = "let f =\n    1\n";
    assert_eq!(
        kinds(src),
        vec![
            k(Keyword::Let),
            id("f"),
            TokenKind::Op(Op::Assign),
            TokenKind::Newline,
            TokenKind::Indent,
            TokenKind::Int(1),
            TokenKind::Newline,
            TokenKind::Dedent,
        ],
    );
}

#[test]
fn nested_indent_emits_two_indents_and_two_dedents() {
    let src = "a\n    b\n        c\n";
    assert_eq!(
        kinds(src),
        vec![
            id("a"),
            TokenKind::Newline,
            TokenKind::Indent,
            id("b"),
            TokenKind::Newline,
            TokenKind::Indent,
            id("c"),
            TokenKind::Newline,
            TokenKind::Dedent,
            TokenKind::Dedent,
        ],
    );
}

#[test]
fn partial_dedent_returns_to_outer_block() {
    let src = "a\n    b\n        c\n    d\n";
    assert_eq!(
        kinds(src),
        vec![
            id("a"),
            TokenKind::Newline,
            TokenKind::Indent,
            id("b"),
            TokenKind::Newline,
            TokenKind::Indent,
            id("c"),
            TokenKind::Newline,
            TokenKind::Dedent,
            id("d"),
            TokenKind::Newline,
            TokenKind::Dedent,
        ],
    );
}

#[test]
fn blank_lines_do_not_emit_newlines() {
    let src = "a\n\n\nb\n";
    assert_eq!(
        kinds(src),
        vec![id("a"), TokenKind::Newline, id("b"), TokenKind::Newline],
    );
}

#[test]
fn comment_only_lines_do_not_emit_newlines() {
    let src = "a\n# comment\nb\n";
    assert_eq!(
        kinds(src),
        vec![id("a"), TokenKind::Newline, id("b"), TokenKind::Newline],
    );
}

#[test]
fn newlines_inside_parens_are_suppressed() {
    let src = "(\n    1\n    2\n)\n";
    let toks = kinds(src);
    assert_eq!(
        toks,
        vec![
            TokenKind::Punct(vela_lexer::Punct::LParen),
            TokenKind::Int(1),
            TokenKind::Int(2),
            TokenKind::Punct(vela_lexer::Punct::RParen),
            TokenKind::Newline,
        ],
    );
}

#[test]
fn dedent_to_zero_at_eof_without_trailing_newline() {
    let src = "a\n    b";
    assert_eq!(
        kinds(src),
        vec![
            id("a"),
            TokenKind::Newline,
            TokenKind::Indent,
            id("b"),
            TokenKind::Newline,
            TokenKind::Dedent,
        ],
    );
}
