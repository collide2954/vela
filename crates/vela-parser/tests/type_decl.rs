use vela_parser::{Stmt, Ty, TypeDecl, TypeDeclBody, TypeVariant, parse_stmt};

fn s(src: &str) -> Stmt {
    parse_stmt(src).expect("parses")
}

fn td(name: &str, params: Vec<&str>, body: TypeDeclBody) -> Stmt {
    Stmt::TypeDecl(TypeDecl {
        name: name.into(),
        params: params.into_iter().map(|s| s.to_string()).collect(),
        body,
    })
}

fn v(name: &str, args: Vec<Ty>) -> TypeVariant {
    TypeVariant { name: name.into(), args }
}

fn con(name: &str) -> Ty {
    Ty::Con(name.into())
}

#[test]
fn single_variant_newtype() {
    assert_eq!(
        s("type Email = Email String"),
        td("Email", vec![], TypeDeclBody::Sum(vec![v("Email", vec![con("String")])])),
    );
}

#[test]
fn enum_of_nullary_variants() {
    assert_eq!(
        s("type Color = | Red | Blue | Green"),
        td(
            "Color",
            vec![],
            TypeDeclBody::Sum(vec![v("Red", vec![]), v("Blue", vec![]), v("Green", vec![])]),
        ),
    );
}

#[test]
fn sum_with_payload() {
    assert_eq!(
        s("type Shape = | Circle Float | Square Float"),
        td(
            "Shape",
            vec![],
            TypeDeclBody::Sum(vec![
                v("Circle", vec![con("Float")]),
                v("Square", vec![con("Float")]),
            ]),
        ),
    );
}

#[test]
fn record_alias() {
    assert_eq!(
        s("type Point = { x : Float, y : Float }"),
        td(
            "Point",
            vec![],
            TypeDeclBody::Alias(Ty::Record(vec![
                ("x".into(), con("Float")),
                ("y".into(), con("Float")),
            ])),
        ),
    );
}

#[test]
fn parametric_type_with_type_variable() {
    let stmt = s("type Option 'a = | None | Some 'a");
    if let Stmt::TypeDecl(decl) = stmt {
        assert_eq!(decl.name, "Option");
        assert_eq!(decl.params, vec!["a".to_string()]);
        if let TypeDeclBody::Sum(variants) = decl.body {
            assert_eq!(variants.len(), 2);
            assert_eq!(variants[0].name, "None");
            assert_eq!(variants[0].args, vec![]);
            assert_eq!(variants[1].name, "Some");
            assert_eq!(variants[1].args, vec![Ty::Var("a".into())]);
        } else {
            panic!("expected sum body");
        }
    } else {
        panic!("expected type decl");
    }
}

#[test]
fn indented_sum_body() {
    let stmt = s("type Color =\n    | Red\n    | Blue\n    | Green");
    if let Stmt::TypeDecl(decl) = stmt {
        assert_eq!(decl.name, "Color");
        if let TypeDeclBody::Sum(variants) = decl.body {
            assert_eq!(variants.len(), 3);
        } else {
            panic!("expected sum");
        }
    } else {
        panic!("expected type decl");
    }
}
