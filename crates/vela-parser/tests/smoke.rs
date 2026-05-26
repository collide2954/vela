use vela_parser::{ImportKind, Stmt, TypeDeclBody, parse_program};

#[test]
fn realistic_module_parses() {
    let src = r#"import std.stats

let standardize xs =
    let m = mean xs
    let s = std xs
    xs |> map (fn x -> (x - m) / s)

let load path =
    let raw = read_file path?
    let df = parse_csv raw?
    Ok df

type Shape =
    | Circle Float
    | Square Float

let area shape =
    match shape with
    | Circle r -> 3.14 * r * r
    | Square s -> s * s
"#;
    let program = parse_program(src).expect("realistic program parses");
    assert_eq!(program.stmts.len(), 5);

    match &program.stmts[0] {
        Stmt::Import { path, kind, public } => {
            assert_eq!(path, &vec!["std".to_string(), "stats".to_string()]);
            assert_eq!(kind, &ImportKind::All);
            assert!(!public);
        }
        other => panic!("expected import, got {other:?}"),
    }

    match &program.stmts[1] {
        Stmt::Let { name, params, .. } => {
            assert_eq!(name, "standardize");
            assert_eq!(params.iter().map(|p| p.simple_name().unwrap()).collect::<Vec<_>>(), vec!["xs"]);
        }
        other => panic!("expected let standardize, got {other:?}"),
    }

    match &program.stmts[2] {
        Stmt::Let { name, params, .. } => {
            assert_eq!(name, "load");
            assert_eq!(params.iter().map(|p| p.simple_name().unwrap()).collect::<Vec<_>>(), vec!["path"]);
        }
        other => panic!("expected let load, got {other:?}"),
    }

    match &program.stmts[3] {
        Stmt::TypeDecl(decl) => {
            assert_eq!(decl.name, "Shape");
            if let TypeDeclBody::Sum(variants) = &decl.body {
                assert_eq!(variants.len(), 2);
                assert_eq!(variants[0].name, "Circle");
                assert_eq!(variants[1].name, "Square");
            } else {
                panic!("expected sum body");
            }
        }
        other => panic!("expected type decl, got {other:?}"),
    }

    match &program.stmts[4] {
        Stmt::Let { name, params, .. } => {
            assert_eq!(name, "area");
            assert_eq!(params.iter().map(|p| p.simple_name().unwrap()).collect::<Vec<_>>(), vec!["shape"]);
        }
        other => panic!("expected let area, got {other:?}"),
    }
}

#[test]
fn dataframe_pipeline_parses() {
    let src = r#"let stats =
    df
    |> group_by :species
    |> summarize { mu = mean (col :petal_length) }
"#;
    let program = parse_program(src).expect("dataframe pipeline parses");
    assert_eq!(program.stmts.len(), 1);
    if let Stmt::Let { name, .. } = &program.stmts[0] {
        assert_eq!(name, "stats");
    } else {
        panic!("expected let stats");
    }
}

#[test]
fn for_loop_with_mutation_parses() {
    let src = r#"var total = 0
for x in xs:
    total <- total + x
"#;
    let program = parse_program(src).expect("for-loop program parses");
    assert_eq!(program.stmts.len(), 2);
    assert!(matches!(program.stmts[0], Stmt::Var { .. }));
    assert!(matches!(program.stmts[1], Stmt::For { .. }));
}

#[test]
fn record_and_field_access_parse() {
    let src = r#"let p = { x = 1.0, y = 2.0 }
let dist = p.x + p.y
"#;
    let program = parse_program(src).expect("records parse");
    assert_eq!(program.stmts.len(), 2);
}

#[test]
fn type_with_parameters_and_match_parses() {
    let src = r#"type Option 'a =
    | None
    | Some 'a

let value_or v default =
    match v with
    | Some x -> x
    | None -> default
"#;
    let program = parse_program(src).expect("parametric type parses");
    assert_eq!(program.stmts.len(), 2);
}
