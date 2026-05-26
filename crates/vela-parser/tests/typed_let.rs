use vela_parser::{Expr, Lit, Param, Stmt, Ty, parse_stmt};

fn s(src: &str) -> Stmt {
    parse_stmt(src).expect("parses")
}

fn con(n: &str) -> Ty {
    Ty::Con(n.into())
}

#[test]
fn let_binding_with_return_type_annotation() {
    assert_eq!(
        s("let m : Float = mean xs"),
        Stmt::Let {
            name: "m".into(),
            params: vec![],
            return_ty: Some(con("Float")),
            body: Expr::App(
                Box::new(Expr::Var("mean".into())),
                Box::new(Expr::Var("xs".into())),
            ),
            recursive: false,
        },
    );
}

#[test]
fn let_function_with_typed_parameter() {
    assert_eq!(
        s("let id (x : Int) = x"),
        Stmt::Let {
            name: "id".into(),
            params: vec![Param { pat: vela_parser::Pat::Var("x".into()), ty: Some(con("Int")) }],
            return_ty: None,
            body: Expr::Var("x".into()),
            recursive: false,
        },
    );
}

#[test]
fn let_function_with_typed_param_and_return_type() {
    let stmt = s("let standardize (xs : [Float]) : [Float] = xs");
    if let Stmt::Let { params, return_ty, .. } = stmt {
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].simple_name().unwrap(), "xs");
        assert_eq!(params[0].ty, Some(Ty::Series(Box::new(con("Float")))));
        assert_eq!(return_ty, Some(Ty::Series(Box::new(con("Float")))));
    } else {
        panic!("expected let");
    }
}

#[test]
fn var_with_type_annotation() {
    assert_eq!(
        s("var counter : Int = 0"),
        Stmt::Var {
            name: "counter".into(),
            ty: Some(con("Int")),
            body: Expr::Lit(Lit::Int(0)),
        },
    );
}

#[test]
fn mixed_typed_and_untyped_params() {
    let stmt = s("let f x (y : Int) z = x");
    if let Stmt::Let { params, .. } = stmt {
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].simple_name().unwrap(), "x");
        assert_eq!(params[0].ty, None);
        assert_eq!(params[1].simple_name().unwrap(), "y");
        assert_eq!(params[1].ty, Some(con("Int")));
        assert_eq!(params[2].simple_name().unwrap(), "z");
        assert_eq!(params[2].ty, None);
    } else {
        panic!("expected let");
    }
}

#[test]
fn untyped_let_still_works() {
    assert_eq!(
        s("let x = 1"),
        Stmt::Let {
            name: "x".into(),
            params: vec![],
            return_ty: None,
            body: Expr::Lit(Lit::Int(1)),
            recursive: false,
        },
    );
}
