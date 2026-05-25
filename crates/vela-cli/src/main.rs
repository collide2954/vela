use std::env;
use std::fs;
use std::process::ExitCode;

use vela_diag::Diagnostic;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let Some(cmd) = args.get(1) else {
        println!("vela {}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("usage: vela <subcommand> [args]");
        println!("subcommands:");
        println!("  check FILE   type-check a single .vela file");
        return ExitCode::SUCCESS;
    };

    match cmd.as_str() {
        "check" => {
            let Some(path) = args.get(2) else {
                eprintln!("usage: vela check FILE");
                return ExitCode::from(2);
            };
            let source = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("cannot read {path}: {e}");
                    return ExitCode::from(1);
                }
            };
            check_one(path, &source)
        }
        "--version" | "-V" => {
            println!("vela {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        other => {
            eprintln!("unknown subcommand: {other}");
            ExitCode::from(2)
        }
    }
}

fn check_one(path: &str, source: &str) -> ExitCode {
    match vela_parser::parse_program(source) {
        Ok(_) => {}
        Err(e) => {
            let mut diag = Diagnostic::error(e.message).with_path(path);
            if let Some(span) = e.span {
                diag = diag.with_span(span);
            }
            eprint!("{}", diag.render(source));
            return ExitCode::from(1);
        }
    }
    match vela_check::check_program(source) {
        Ok(_) => {
            println!("ok");
            ExitCode::SUCCESS
        }
        Err(e) => {
            let diag = Diagnostic::error(e.message).with_path(path);
            eprint!("{}", diag.render(source));
            ExitCode::from(1)
        }
    }
}
