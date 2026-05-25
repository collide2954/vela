use vela_lexer::{Keyword, TokenKind, lex};

fn kinds(src: &str) -> Vec<TokenKind> {
    lex(src).map(|t| t.kind).collect()
}

#[test]
fn simple_ident() {
    assert_eq!(kinds("foo"), vec![TokenKind::Ident("foo".into())]);
}

#[test]
fn ident_with_underscore_and_digits() {
    assert_eq!(kinds("snake_case_42"), vec![TokenKind::Ident("snake_case_42".into())]);
}

#[test]
fn ident_starting_with_underscore() {
    assert_eq!(kinds("_x"), vec![TokenKind::Ident("_x".into())]);
}

#[test]
fn wildcard_ident_underscore_alone() {
    assert_eq!(kinds("_"), vec![TokenKind::Ident("_".into())]);
}

#[test]
fn type_name_camel_case() {
    assert_eq!(kinds("DataFrame"), vec![TokenKind::Ident("DataFrame".into())]);
}

#[test]
fn keyword_let() {
    assert_eq!(kinds("let"), vec![TokenKind::Keyword(Keyword::Let)]);
}

#[test]
fn keyword_var() {
    assert_eq!(kinds("var"), vec![TokenKind::Keyword(Keyword::Var)]);
}

#[test]
fn keyword_fn() {
    assert_eq!(kinds("fn"), vec![TokenKind::Keyword(Keyword::Fn)]);
}

#[test]
fn keyword_match_with_when() {
    assert_eq!(
        kinds("match with when"),
        vec![
            TokenKind::Keyword(Keyword::Match),
            TokenKind::Keyword(Keyword::With),
            TokenKind::Keyword(Keyword::When),
        ],
    );
}

#[test]
fn keyword_and_or_not() {
    assert_eq!(
        kinds("and or not"),
        vec![
            TokenKind::Keyword(Keyword::And),
            TokenKind::Keyword(Keyword::Or),
            TokenKind::Keyword(Keyword::Not),
        ],
    );
}

#[test]
fn keyword_app_input_output() {
    assert_eq!(
        kinds("app input output"),
        vec![
            TokenKind::Keyword(Keyword::App),
            TokenKind::Keyword(Keyword::Input),
            TokenKind::Keyword(Keyword::Output),
        ],
    );
}

#[test]
fn keyword_tests_test_prop() {
    assert_eq!(
        kinds("tests test prop"),
        vec![
            TokenKind::Keyword(Keyword::Tests),
            TokenKind::Keyword(Keyword::Test),
            TokenKind::Keyword(Keyword::Prop),
        ],
    );
}

#[test]
fn boolean_literals_are_bool_tokens() {
    assert_eq!(
        kinds("true false"),
        vec![TokenKind::Bool(true), TokenKind::Bool(false)],
    );
}

#[test]
fn ident_that_starts_with_keyword_prefix() {
    assert_eq!(kinds("letter"), vec![TokenKind::Ident("letter".into())]);
}

#[test]
fn keyword_is_case_sensitive() {
    assert_eq!(kinds("Let"), vec![TokenKind::Ident("Let".into())]);
}
