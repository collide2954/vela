use vela_check::{Type, check_program};

#[test]
fn dataframe_no_schema_is_loose() {
    let src = "let df = {| x : [1, 2, 3] |}\ndf.x";
    let t = check_program(src).expect("ok");
    assert!(matches!(t, Type::Series(_)));
}

#[test]
fn dataframe_with_schema_typed_column_access() {
    let src = "let df : DataFrame[{ x : Float, y : Int }] = {| x : [1.0, 2.0], y : [1, 2] |}\ndf.x";
    let t = check_program(src).expect("ok");
    assert_eq!(
        t,
        Type::Series(Box::new(Type::Option(Box::new(Type::Float)))),
    );
}

#[test]
fn dataframe_with_schema_int_column() {
    let src = "let df : DataFrame[{ x : Float, y : Int }] = {| x : [1.0, 2.0], y : [1, 2] |}\ndf.y";
    let t = check_program(src).expect("ok");
    assert_eq!(t, Type::Series(Box::new(Type::Option(Box::new(Type::Int)))),);
}

#[test]
fn dataframe_missing_column_error() {
    let src = "let df : DataFrame[{ x : Float }] = {| x : [1.0] |}\ndf.bogus";
    assert!(check_program(src).is_err());
}
