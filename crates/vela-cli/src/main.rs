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
        "run" => {
            let Some(path) = args.get(2) else {
                eprintln!("usage: vela run FILE");
                return ExitCode::from(2);
            };
            let source = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("cannot read {path}: {e}");
                    return ExitCode::from(1);
                }
            };
            run_one(path, &source)
        }
        "test" => {
            let Some(path) = args.get(2) else {
                eprintln!("usage: vela test FILE");
                return ExitCode::from(2);
            };
            let source = match fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("cannot read {path}: {e}");
                    return ExitCode::from(1);
                }
            };
            test_one(path, &source)
        }
        "explain" => {
            let Some(code) = args.get(2) else {
                eprintln!("usage: vela explain CODE");
                return ExitCode::from(2);
            };
            match vela_check::explain(code) {
                Some(text) => {
                    println!("{text}");
                    ExitCode::SUCCESS
                }
                None => {
                    eprintln!("no explanation available for {code}");
                    ExitCode::from(1)
                }
            }
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

fn test_one(path: &str, source: &str) -> ExitCode {
    match vela_parser::parse_program(source) {
        Ok(_) => {}
        Err(e) => {
            let mut diag = Diagnostic::error(e.message)
                .with_path(path)
                .with_code(e.code);
            if let Some(span) = e.span {
                diag = diag.with_span(span);
            }
            eprint!("{}", diag.render(source));
            return ExitCode::from(1);
        }
    }
    match vela_check::check_program_with_warnings(source) {
        Ok((_, warnings)) => {
            for w in &warnings {
                let diag = Diagnostic::warning(&w.message)
                    .with_path(path)
                    .with_code(w.code);
                eprint!("{}", diag.render(source));
            }
        }
        Err(e) => {
            let diag = Diagnostic::error(e.message)
                .with_path(path)
                .with_code(e.code);
            eprint!("{}", diag.render(source));
            return ExitCode::from(1);
        }
    }
    match vela_eval::run_tests(source) {
        Ok(reports) => {
            let total = reports.len();
            let passed = reports.iter().filter(|r| r.passed).count();
            for r in &reports {
                if r.passed {
                    println!("ok    {}", r.name);
                } else {
                    println!("fail  {}", r.name);
                    if let Some(m) = &r.message {
                        println!("      {m}");
                    }
                }
            }
            println!();
            println!("{passed}/{total} tests passed");
            if passed == total {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Err(e) => {
            eprintln!("error: {}", e.message);
            ExitCode::from(1)
        }
    }
}

fn run_one(path: &str, source: &str) -> ExitCode {
    match vela_parser::parse_program(source) {
        Ok(_) => {}
        Err(e) => {
            let mut diag = Diagnostic::error(e.message)
                .with_path(path)
                .with_code(e.code);
            if let Some(span) = e.span {
                diag = diag.with_span(span);
            }
            eprint!("{}", diag.render(source));
            return ExitCode::from(1);
        }
    }
    match vela_check::check_program_with_warnings(source) {
        Ok((_, warnings)) => {
            for w in &warnings {
                let diag = Diagnostic::warning(&w.message)
                    .with_path(path)
                    .with_code(w.code);
                eprint!("{}", diag.render(source));
            }
        }
        Err(e) => {
            let diag = Diagnostic::error(e.message)
                .with_path(path)
                .with_code(e.code);
            eprint!("{}", diag.render(source));
            return ExitCode::from(1);
        }
    }
    match vela_eval::run(source) {
        Ok(v) => {
            println!("{}", vela_eval::show(&v));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("runtime error: {}", e.message);
            ExitCode::from(1)
        }
    }
}

fn check_one(path: &str, source: &str) -> ExitCode {
    match vela_parser::parse_program(source) {
        Ok(_) => {}
        Err(e) => {
            let mut diag = Diagnostic::error(e.message)
                .with_path(path)
                .with_code(e.code);
            if let Some(span) = e.span {
                diag = diag.with_span(span);
            }
            eprint!("{}", diag.render(source));
            return ExitCode::from(1);
        }
    }
    match vela_check::check_program_with_warnings(source) {
        Ok((_, warnings)) => {
            for w in &warnings {
                let diag = Diagnostic::warning(&w.message)
                    .with_path(path)
                    .with_code(w.code);
                eprint!("{}", diag.render(source));
            }
            println!("ok");
            ExitCode::SUCCESS
        }
        Err(e) => {
            let diag = Diagnostic::error(e.message)
                .with_path(path)
                .with_code(e.code);
            eprint!("{}", diag.render(source));
            ExitCode::from(1)
        }
    }
}
