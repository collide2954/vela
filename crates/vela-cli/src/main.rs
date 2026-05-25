use std::env;
use std::fs;
use std::process::ExitCode;

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
            match vela_check::check_program(&source) {
                Ok(_) => {
                    println!("ok");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("error: {}", e.message);
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
