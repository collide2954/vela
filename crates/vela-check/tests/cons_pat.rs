use vela_check::{Type, check_expr};

fn t(src: &str) -> Type {
    check_expr(src).expect("type-checks")
}

#[test]
fn match_option_some_none() {
    assert_eq!(t("match Some 5 with | None -> 0 | Some x -> x"), Type::Int,);
}

#[test]
fn match_result_ok_err() {
    assert_eq!(t(r#"match Ok 5 with | Ok v -> v | Err _ -> 0"#), Type::Int,);
}

#[test]
fn match_none_branch_yields_default() {
    assert_eq!(t("match None with | None -> 0 | Some _ -> 1"), Type::Int,);
}

#[test]
fn match_constructor_payload_drives_arm_body() {
    assert_eq!(
        t(r#"match Some "abc" with | None -> "" | Some s -> s"#),
        Type::String,
    );
}

#[test]
fn arm_body_types_must_match() {
    let e = check_expr(r#"match Some 5 with | None -> "no" | Some x -> x"#)
        .unwrap_err()
        .message;
    assert!(e.contains("Int") && e.contains("String"));
}
