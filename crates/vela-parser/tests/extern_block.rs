use vela_parser::{Stmt, Ty, parse_stmt};

fn s(src: &str) -> Stmt {
    parse_stmt(src).expect("parses")
}

#[test]
fn simple_extern_c_with_one_signature() {
    let stmt = s(r#"extern "C" =
    fn add (x : Int) (y : Int) : Int"#);
    if let Stmt::Extern { abi, signatures } = stmt {
        assert_eq!(abi, "C");
        assert_eq!(signatures.len(), 1);
        assert_eq!(signatures[0].name, "add");
        assert_eq!(signatures[0].return_ty, Ty::Con("Int".into()));
    } else {
        panic!("expected extern block");
    }
}

#[test]
fn extern_with_two_signatures() {
    let stmt = s(r#"extern "C" =
    fn a (x : Int) : Int
    fn b (x : Int) : Int"#);
    if let Stmt::Extern { signatures, .. } = stmt {
        assert_eq!(signatures.len(), 2);
        assert_eq!(signatures[0].name, "a");
        assert_eq!(signatures[1].name, "b");
    } else {
        panic!("expected extern block");
    }
}
