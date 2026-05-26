use vela_parser::{ImportKind, Stmt, parse_stmt};

fn s(src: &str) -> Stmt {
    parse_stmt(src).expect("parses")
}

fn imp(path: Vec<&str>, kind: ImportKind) -> Stmt {
    Stmt::Import {
        path: path.into_iter().map(|s| s.to_string()).collect(),
        kind,
        public: false,
    }
}

#[test]
fn import_single_module() {
    assert_eq!(s("import stats"), imp(vec!["stats"], ImportKind::All));
}

#[test]
fn import_dotted_path() {
    assert_eq!(
        s("import stats.dist"),
        imp(vec!["stats", "dist"], ImportKind::All)
    );
}

#[test]
fn import_deep_path() {
    assert_eq!(
        s("import std.stats.dist"),
        imp(vec!["std", "stats", "dist"], ImportKind::All),
    );
}

#[test]
fn import_selective_items() {
    assert_eq!(
        s("import stats.dist (Normal, T)"),
        imp(
            vec!["stats", "dist"],
            ImportKind::Items(vec!["Normal".into(), "T".into()]),
        ),
    );
}

#[test]
fn import_single_item() {
    assert_eq!(
        s("import stats (mean)"),
        imp(vec!["stats"], ImportKind::Items(vec!["mean".into()])),
    );
}

#[test]
fn import_with_alias() {
    assert_eq!(
        s("import frame as f"),
        imp(vec!["frame"], ImportKind::Alias("f".into())),
    );
}

#[test]
fn import_dotted_with_alias() {
    assert_eq!(
        s("import std.http as h"),
        imp(vec!["std", "http"], ImportKind::Alias("h".into())),
    );
}

#[test]
fn pub_import_is_let_export() {
    // pub import x.y — re-export form
    let stmt = s("pub import foo");
    if let Stmt::Import { path, kind, public } = stmt {
        assert_eq!(path, vec!["foo".to_string()]);
        assert_eq!(kind, ImportKind::All);
        assert!(public);
    } else {
        panic!("expected import");
    }
}
