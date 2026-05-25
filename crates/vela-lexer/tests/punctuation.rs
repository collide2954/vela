use vela_lexer::{Op, Punct, TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
    lex(src).map(|t| t.kind).collect()
}

fn op(o: Op) -> TokenKind {
    TokenKind::Op(o)
}
fn p(x: Punct) -> TokenKind {
    TokenKind::Punct(x)
}

#[test]
fn single_paren_pair() {
    assert_eq!(kinds("()"), vec![p(Punct::LParen), p(Punct::RParen)]);
}

#[test]
fn brackets_and_braces() {
    assert_eq!(
        kinds("[]{}"),
        vec![p(Punct::LBracket), p(Punct::RBracket), p(Punct::LBrace), p(Punct::RBrace)],
    );
}

#[test]
fn comma_semi_colon() {
    assert_eq!(
        kinds(", ; :"),
        vec![p(Punct::Comma), p(Punct::Semi), p(Punct::Colon)],
    );
}

#[test]
fn array_literal_delimiters() {
    assert_eq!(
        kinds("[| 1 ; 2 |]"),
        vec![
            p(Punct::ArrayOpen),
            TokenKind::Int(1),
            p(Punct::Semi),
            TokenKind::Int(2),
            p(Punct::ArrayClose),
        ],
    );
}

#[test]
fn frame_literal_delimiters() {
    assert_eq!(
        kinds("{| 1 |}"),
        vec![p(Punct::FrameOpen), TokenKind::Int(1), p(Punct::FrameClose)],
    );
}

#[test]
fn single_char_arithmetic_ops() {
    assert_eq!(
        kinds("+ - * / %"),
        vec![op(Op::Plus), op(Op::Minus), op(Op::Star), op(Op::Slash), op(Op::Percent)],
    );
}

#[test]
fn caret_and_tilde() {
    assert_eq!(kinds("^ ~"), vec![op(Op::Caret), op(Op::Tilde)]);
}

#[test]
fn comparison_ops() {
    assert_eq!(
        kinds("== != < <= > >="),
        vec![op(Op::Eq), op(Op::NotEq), op(Op::Lt), op(Op::Le), op(Op::Gt), op(Op::Ge)],
    );
}

#[test]
fn assign_vs_equality() {
    assert_eq!(kinds("="), vec![op(Op::Assign)]);
    assert_eq!(kinds("=="), vec![op(Op::Eq)]);
}

#[test]
fn arrows() {
    assert_eq!(kinds("-> <-"), vec![op(Op::RArrow), op(Op::LArrow)]);
}

#[test]
fn pipe_vs_bar() {
    assert_eq!(kinds("|>"), vec![op(Op::Pipe)]);
    assert_eq!(kinds("|"), vec![p(Punct::Bar)]);
}

#[test]
fn plus_plus() {
    assert_eq!(kinds("++"), vec![op(Op::PlusPlus)]);
    assert_eq!(kinds("+ +"), vec![op(Op::Plus), op(Op::Plus)]);
}

#[test]
fn question_mark() {
    assert_eq!(kinds("?"), vec![op(Op::Question)]);
}

#[test]
fn dot_and_ranges() {
    assert_eq!(kinds("."), vec![op(Op::Dot)]);
    assert_eq!(kinds(".."), vec![op(Op::DotDot)]);
    assert_eq!(kinds("..="), vec![op(Op::DotDotEq)]);
}

#[test]
fn type_var_tick() {
    assert_eq!(
        kinds("'a"),
        vec![p(Punct::Tick), TokenKind::Ident("a".into())],
    );
}

#[test]
fn small_program_lexes() {
    let src = "let f = fn x -> x + 1";
    let toks = kinds(src);
    assert_eq!(
        toks,
        vec![
            TokenKind::Keyword(vela_lexer::Keyword::Let),
            TokenKind::Ident("f".into()),
            op(Op::Assign),
            TokenKind::Keyword(vela_lexer::Keyword::Fn),
            TokenKind::Ident("x".into()),
            op(Op::RArrow),
            TokenKind::Ident("x".into()),
            op(Op::Plus),
            TokenKind::Int(1),
        ],
    );
}
