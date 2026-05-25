use vela_check::{Type, check_expr};

fn t(src: &str) -> Type {
    check_expr(src).expect("type-checks")
}

fn opt(inner: Type) -> Type {
    Type::Option(Box::new(inner))
}

#[test]
fn some_of_int_is_option_int() {
    assert_eq!(t("Some 5"), opt(Type::Int));
}

#[test]
fn some_of_string() {
    assert_eq!(t(r#"Some "abc""#), opt(Type::String));
}

#[test]
fn none_alone_is_polymorphic_option() {
    assert!(matches!(t("None"), Type::Option(_)));
}

#[test]
fn ok_of_int_with_polymorphic_error() {
    let result = t("Ok 5");
    if let Type::Result(a, _) = result {
        assert_eq!(*a, Type::Int);
    } else {
        panic!("expected Result");
    }
}

#[test]
fn err_of_string() {
    let result = t(r#"Err "boom""#);
    if let Type::Result(_, e) = result {
        assert_eq!(*e, Type::String);
    } else {
        panic!("expected Result");
    }
}

#[test]
fn option_in_record() {
    assert_eq!(
        t("{ value = Some 1 }"),
        Type::Record(vec![("value".into(), opt(Type::Int))], None),
    );
}

#[test]
fn result_can_be_in_series() {
    let r = t("[Ok 1, Ok 2]");
    assert!(matches!(r, Type::Series(_)));
}

#[test]
fn unify_some_and_none_in_series() {
    // [Some 1, None] should give Series[Option[Int]]
    assert_eq!(
        t("[Some 1, None]"),
        Type::Series(Box::new(opt(Type::Int))),
    );
}
