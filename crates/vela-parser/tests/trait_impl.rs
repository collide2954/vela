use vela_parser::{Expr, ImplBlock, ImplMethod, Param, Stmt, TraitDecl, TraitMethodSig, Ty, parse_stmt};

fn s(src: &str) -> Stmt {
    parse_stmt(src).expect("parses")
}

fn con(n: &str) -> Ty {
    Ty::Con(n.into())
}

#[test]
fn simple_trait_one_method() {
    let stmt = s("trait Show t =\n    fn show (x : t) : String");
    assert_eq!(
        stmt,
        Stmt::TraitDecl(TraitDecl {
            name: "Show".into(),
            type_var: "t".into(),
            methods: vec![TraitMethodSig {
                name: "show".into(),
                params: vec![Param { pat: vela_parser::Pat::Var("x".into()), ty: Some(con("t")) }],
                return_ty: con("String"),
            }],
        }),
    );
}

#[test]
fn trait_with_two_methods() {
    let stmt = s(
        "trait Eq t =\n    fn eq (a : t) (b : t) : Bool\n    fn neq (a : t) (b : t) : Bool",
    );
    if let Stmt::TraitDecl(decl) = stmt {
        assert_eq!(decl.name, "Eq");
        assert_eq!(decl.methods.len(), 2);
        assert_eq!(decl.methods[0].name, "eq");
        assert_eq!(decl.methods[1].name, "neq");
    } else {
        panic!("expected trait decl");
    }
}

#[test]
fn simple_impl_block() {
    let stmt = s("impl Show Float =\n    fn show x = x");
    assert_eq!(
        stmt,
        Stmt::Impl(ImplBlock {
            trait_name: "Show".into(),
            ty: con("Float"),
            methods: vec![ImplMethod {
                name: "show".into(),
                params: vec![Param { pat: vela_parser::Pat::Var("x".into()), ty: None }],
                return_ty: None,
                body: Expr::Var("x".into()),
            }],
        }),
    );
}

#[test]
fn impl_with_typed_method() {
    let stmt = s("impl Show Int =\n    fn show (x : Int) : String = format_int x");
    if let Stmt::Impl(block) = stmt {
        assert_eq!(block.trait_name, "Show");
        assert_eq!(block.ty, con("Int"));
        assert_eq!(block.methods.len(), 1);
        assert_eq!(block.methods[0].name, "show");
        assert_eq!(block.methods[0].return_ty, Some(con("String")));
    } else {
        panic!("expected impl");
    }
}

#[test]
fn impl_for_parametric_type() {
    let stmt = s("impl Show (Option a) =\n    fn show x = x");
    if let Stmt::Impl(block) = stmt {
        assert_eq!(block.trait_name, "Show");
        assert_eq!(
            block.ty,
            Ty::App(Box::new(con("Option")), vec![con("a")]),
        );
    } else {
        panic!("expected impl");
    }
}
