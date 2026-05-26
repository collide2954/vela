//! Read-eval-print loop for the Vela language.

use std::io::{self, BufRead, IsTerminal, Write};

pub fn run() -> i32 {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut check = vela_check::Session::new();
    let mut eval = vela_eval::Session::new();
    let interactive = stdin.is_terminal();
    if interactive {
        println!("vela {} (REPL)", env!("CARGO_PKG_VERSION"));
        println!("blank line submits; Ctrl-D exits");
    }
    let mut buf = String::new();
    let mut handle = stdin.lock();
    loop {
        if interactive {
            let prompt = if buf.is_empty() { "> " } else { "  " };
            print!("{prompt}");
            stdout.flush().ok();
        }
        let mut line = String::new();
        match handle.read_line(&mut line) {
            Ok(0) => {
                if !buf.trim().is_empty() {
                    run_chunk(&mut check, &mut eval, &buf);
                }
                if interactive {
                    println!();
                }
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("read error: {e}");
                return 1;
            }
        }
        if line.trim().is_empty() {
            if !buf.trim().is_empty() {
                run_chunk(&mut check, &mut eval, &buf);
                buf.clear();
            }
            continue;
        }
        buf.push_str(&line);
        if !interactive && looks_complete(&buf) && !buf_continues(&buf) {
            run_chunk(&mut check, &mut eval, &buf);
            buf.clear();
        }
    }
    0
}

fn looks_complete(src: &str) -> bool {
    !src.trim().is_empty() && vela_parser::parse_program(src).is_ok()
}

fn buf_continues(src: &str) -> bool {
    let last = src.lines().rev().find(|l| !l.trim().is_empty());
    match last {
        None => false,
        Some(line) => {
            let trimmed = line.trim_end();
            trimmed.ends_with('=')
                || trimmed.ends_with(':')
                || trimmed.ends_with('|')
                || trimmed.starts_with("    ")
                || trimmed.starts_with('\t')
        }
    }
}

fn run_chunk(check: &mut vela_check::Session, eval: &mut vela_eval::Session, src: &str) {
    match check.check_str(src) {
        Ok((_, warnings)) => {
            for w in &warnings {
                eprintln!("warning[{}]: {}", w.code, w.message);
            }
        }
        Err(e) => {
            eprintln!("error[{}]: {}", e.code, e.message);
            return;
        }
    }
    match eval.eval_str(src) {
        Ok(v) => match v {
            vela_eval::Value::Unit => {}
            other => println!("{}", vela_eval::show(&other)),
        },
        Err(e) => eprintln!("runtime error: {}", e.message),
    }
}
