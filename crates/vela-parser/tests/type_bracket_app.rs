use vela_parser::{Stmt, Ty, parse_stmt};

fn parse_let_annotation_type(src: &str) -> Ty {
    if let Stmt::Let { return_ty: Some(t), .. } = parse_stmt(src).expect("parses") {
        t
    } else {
        panic!("expected let with return type annotation");
    }
}

fn parse_var_annotation_type(src: &str) -> Ty {
    if let Stmt::Var { ty: Some(t), .. } = parse_stmt(src).expect("parses") {
        t
    } else {
        panic!("expected var with annotation");
    }
}

#[test]
fn option_t_bracket() {
    assert_eq!(
        parse_let_annotation_type("let m : Option[Int] = m"),
        Ty::App(Box::new(Ty::Con("Option".into())), vec![Ty::Con("Int".into())]),
    );
}

#[test]
fn result_t_e_bracket() {
    assert_eq!(
        parse_let_annotation_type("let r : Result[Int, String] = r"),
        Ty::App(
            Box::new(Ty::Con("Result".into())),
            vec![Ty::Con("Int".into()), Ty::Con("String".into())],
        ),
    );
}

#[test]
fn array_with_dimension_literal_skipped() {
    assert_eq!(
        parse_let_annotation_type("let m : Array[Float, 2] = m"),
        Ty::App(
            Box::new(Ty::Con("Array".into())),
            vec![Ty::Con("Float".into()), Ty::Con("_dim".into())],
        ),
    );
}

#[test]
fn nested_option_of_series() {
    assert_eq!(
        parse_let_annotation_type("let r : Option[[Int]] = r"),
        Ty::App(
            Box::new(Ty::Con("Option".into())),
            vec![Ty::Series(Box::new(Ty::Con("Int".into())))],
        ),
    );
}

#[test]
fn var_bracket_annotation_too() {
    assert_eq!(
        parse_var_annotation_type("var x : Option[Int] = x"),
        Ty::App(Box::new(Ty::Con("Option".into())), vec![Ty::Con("Int".into())]),
    );
}
