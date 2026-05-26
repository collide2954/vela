//! Package scaffolding and manifest handling for Vela projects.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Bin,
    Lib,
}

#[derive(Debug)]
pub enum NewError {
    InvalidName(String),
    AlreadyExists(PathBuf),
    Io(io::Error),
}

impl std::fmt::Display for NewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NewError::InvalidName(n) => write!(f, "invalid package name `{n}`"),
            NewError::AlreadyExists(p) => write!(f, "{} already exists", p.display()),
            NewError::Io(e) => write!(f, "{e}"),
        }
    }
}

impl From<io::Error> for NewError {
    fn from(e: io::Error) -> Self {
        NewError::Io(e)
    }
}

pub fn new_project(root: &Path, name: &str, kind: Kind) -> Result<PathBuf, NewError> {
    validate_name(name)?;
    let dir = root.join(name);
    if dir.exists() {
        return Err(NewError::AlreadyExists(dir));
    }
    fs::create_dir_all(dir.join("src"))?;
    fs::write(dir.join("vela.toml"), manifest(name))?;
    fs::write(dir.join(".gitignore"), gitignore())?;
    match kind {
        Kind::Bin => {
            fs::write(dir.join("src").join("main.vela"), bin_starter())?;
        }
        Kind::Lib => {
            fs::write(dir.join("src").join("lib.vela"), lib_starter(name))?;
            fs::create_dir_all(dir.join("tests"))?;
            fs::write(dir.join("tests").join("smoke.vela"), lib_test_starter(name))?;
        }
    }
    Ok(dir)
}

fn validate_name(name: &str) -> Result<(), NewError> {
    if name.is_empty() {
        return Err(NewError::InvalidName(name.into()));
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_lowercase() || first == '_') {
        return Err(NewError::InvalidName(name.into()));
    }
    for c in chars {
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            return Err(NewError::InvalidName(name.into()));
        }
    }
    Ok(())
}

fn manifest(name: &str) -> String {
    format!("[package]\nname    = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2026\"\n\n[deps]\n")
}

fn gitignore() -> String {
    "/.vela/\n".into()
}

fn bin_starter() -> String {
    "println \"hello, vela\"\n".into()
}

fn lib_starter(name: &str) -> String {
    format!(
        "pub let greeting who = \"hello, \" ++ who\n\n# {name} is a library; entry points go here.\n"
    )
}

fn lib_test_starter(_name: &str) -> String {
    "tests =\n    test \"greeting\" =\n        assert (greeting \"world\" == \"hello, world\")\n"
        .into()
}

pub fn is_valid_name(name: &str) -> bool {
    validate_name(name).is_ok()
}
