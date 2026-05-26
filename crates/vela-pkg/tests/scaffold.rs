use std::fs;
use vela_pkg::{Kind, new_project};

fn tempdir(label: &str) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("vela-pkg-test-{}-{}", std::process::id(), label));
    if p.exists() {
        fs::remove_dir_all(&p).unwrap();
    }
    fs::create_dir_all(&p).unwrap();
    p
}

#[test]
fn new_binary_layout() {
    let root = tempdir("new_binary_layout");
    let dir = new_project(&root, "demo", Kind::Bin).expect("creates");
    assert!(dir.join("vela.toml").exists());
    assert!(dir.join(".gitignore").exists());
    assert!(dir.join("src/main.vela").exists());
    let manifest = fs::read_to_string(dir.join("vela.toml")).unwrap();
    assert!(manifest.contains("name    = \"demo\""));
}

#[test]
fn new_library_layout() {
    let root = tempdir("new_library_layout");
    let dir = new_project(&root, "lib_demo", Kind::Lib).expect("creates");
    assert!(dir.join("src/lib.vela").exists());
    assert!(dir.join("tests/smoke.vela").exists());
}

#[test]
fn rejects_uppercase_name() {
    let root = tempdir("rejects_uppercase_name");
    let result = new_project(&root, "Bad", Kind::Bin);
    assert!(result.is_err());
}

#[test]
fn rejects_existing_dir() {
    let root = tempdir("rejects_existing_dir");
    let dir = root.join("dupe");
    fs::create_dir_all(&dir).unwrap();
    let result = new_project(&root, "dupe", Kind::Bin);
    assert!(matches!(result, Err(vela_pkg::NewError::AlreadyExists(_))));
}
