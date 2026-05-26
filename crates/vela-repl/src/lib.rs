//! Read-eval-print loop for the Vela language.

use rustyline::DefaultEditor;
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use std::fs;
use std::path::PathBuf;

pub fn run() -> i32 {
    let mut sess = Session::new();
    let interactive = std::io::IsTerminal::is_terminal(&std::io::stdin());
    if !interactive {
        return run_piped(&mut sess);
    }
    let editor = match DefaultEditor::new() {
        Ok(e) => Some(e),
        Err(e) => {
            eprintln!("warning: line editor unavailable ({e}); falling back to plain input");
            None
        }
    };
    if let Some(mut ed) = editor {
        configure_editor(&mut ed);
        let hist = history_path();
        if let Some(path) = hist.as_ref() {
            let _ = ed.load_history(path);
        }
        let exit = run_interactive(&mut sess, &mut ed);
        if let Some(path) = hist.as_ref() {
            let _ = ed.save_history(path);
        }
        exit
    } else {
        run_piped(&mut sess)
    }
}

fn configure_editor(ed: &mut DefaultEditor) {
    ed.set_auto_add_history(true);
}

fn run_interactive(sess: &mut Session, ed: &mut DefaultEditor) -> i32 {
    print_banner();
    let mut buf = String::new();
    loop {
        let prompt = if buf.is_empty() { "vela> " } else { "...  " };
        match ed.readline(prompt) {
            Ok(line) => {
                if buf.is_empty() {
                    if let Some(meta) = parse_meta(&line) {
                        let cont = run_meta(sess, meta);
                        if !cont {
                            return 0;
                        }
                        continue;
                    }
                }
                if line.trim().is_empty() {
                    if !buf.trim().is_empty() {
                        let chunk = std::mem::take(&mut buf);
                        run_chunk(sess, &chunk);
                    }
                    continue;
                }
                if !buf.is_empty() {
                    buf.push('\n');
                }
                buf.push_str(&line);
                if input_complete(&buf) {
                    let chunk = std::mem::take(&mut buf);
                    run_chunk(sess, &chunk);
                }
            }
            Err(ReadlineError::Interrupted) => {
                buf.clear();
                println!("(canceled)");
            }
            Err(ReadlineError::Eof) => {
                if !buf.trim().is_empty() {
                    let chunk = std::mem::take(&mut buf);
                    run_chunk(sess, &chunk);
                }
                return 0;
            }
            Err(e) => {
                eprintln!("readline error: {e}");
                return 1;
            }
        }
    }
}

fn run_piped(sess: &mut Session) -> i32 {
    use std::io::{BufRead, BufReader};
    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());
    let mut buf = String::new();
    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("read error: {e}");
                return 1;
            }
        };
        if buf.is_empty() {
            if let Some(meta) = parse_meta(&line) {
                if !run_meta(sess, meta) {
                    return 0;
                }
                continue;
            }
        }
        if line.trim().is_empty() {
            if !buf.trim().is_empty() {
                let chunk = std::mem::take(&mut buf);
                run_chunk(sess, &chunk);
            }
            continue;
        }
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(&line);
        if input_complete(&buf) {
            let chunk = std::mem::take(&mut buf);
            run_chunk(sess, &chunk);
        }
    }
    if !buf.trim().is_empty() {
        let chunk = std::mem::take(&mut buf);
        run_chunk(sess, &chunk);
    }
    0
}

fn print_banner() {
    println!("vela {} REPL", env!("CARGO_PKG_VERSION"));
    println!("type :help for commands, :quit to exit");
}

pub struct Session {
    check: vela_check::Session,
    eval: vela_eval::Session,
}

impl Session {
    pub fn new() -> Self {
        Self {
            check: vela_check::Session::new(),
            eval: vela_eval::Session::new(),
        }
    }

    pub fn reset(&mut self) {
        self.check = vela_check::Session::new();
        self.eval = vela_eval::Session::new();
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
enum Meta<'a> {
    Help,
    Quit,
    Reset,
    TypeOf(&'a str),
    Load(&'a str),
}

fn parse_meta(line: &str) -> Option<Meta<'_>> {
    let line = line.trim();
    if !line.starts_with(':') {
        return None;
    }
    let mut parts = line[1..].splitn(2, char::is_whitespace);
    let cmd = parts.next()?;
    let rest = parts.next().unwrap_or("").trim();
    match cmd {
        "help" | "h" | "?" => Some(Meta::Help),
        "quit" | "q" | "exit" => Some(Meta::Quit),
        "reset" => Some(Meta::Reset),
        "type" | "t" => Some(Meta::TypeOf(rest)),
        "load" | "l" => Some(Meta::Load(rest)),
        _ => {
            eprintln!("unknown command: :{cmd} (try :help)");
            Some(Meta::Help)
        }
    }
}

fn run_meta(sess: &mut Session, meta: Meta<'_>) -> bool {
    match meta {
        Meta::Help => {
            print_help();
            true
        }
        Meta::Quit => false,
        Meta::Reset => {
            sess.reset();
            println!("session reset");
            true
        }
        Meta::TypeOf(src) => {
            if src.is_empty() {
                eprintln!("usage: :type EXPR");
            } else {
                match sess.check.type_of(src) {
                    Ok(t) => println!("{} : {}", src, t.display()),
                    Err(e) => eprintln!("error[{}]: {}", e.code, e.message),
                }
            }
            true
        }
        Meta::Load(path) => {
            if path.is_empty() {
                eprintln!("usage: :load PATH");
                return true;
            }
            match fs::read_to_string(path) {
                Ok(source) => run_chunk(sess, &source),
                Err(e) => eprintln!("cannot read {path}: {e}"),
            }
            true
        }
    }
}

fn print_help() {
    println!("REPL commands:");
    println!("  :help                show this message");
    println!("  :quit | :exit        leave the REPL");
    println!("  :reset               clear all bindings");
    println!("  :type EXPR           show the inferred type of EXPR");
    println!("  :load PATH           read PATH and execute it in this session");
    println!();
    println!("Editing:");
    println!("  Enter                submit if input is complete, otherwise continue");
    println!("  blank line           force-submit the current buffer");
    println!("  Ctrl-C               cancel current input");
    println!("  Ctrl-D               exit");
}

fn input_complete(src: &str) -> bool {
    if src.trim().is_empty() {
        return false;
    }
    let last = match src.lines().rev().find(|l| !l.trim().is_empty()) {
        Some(l) => l,
        None => return false,
    };
    let trimmed = last.trim_end();
    if trimmed.ends_with('=') || trimmed.ends_with(':') {
        return false;
    }
    if last.starts_with(' ') || last.starts_with('\t') {
        return false;
    }
    true
}

fn run_chunk(sess: &mut Session, src: &str) {
    let (ty, warnings) = match sess.check.check_str(src) {
        Ok(out) => out,
        Err(e) => {
            eprintln!("error[{}]: {}", e.code, e.message);
            return;
        }
    };
    for w in &warnings {
        eprintln!("warning[{}]: {}", w.code, w.message);
    }
    match sess.eval.eval_str(src) {
        Ok(v) => match v {
            vela_eval::Value::Unit => {}
            other => println!("{} : {}", vela_eval::show(&other), ty.display()),
        },
        Err(e) => eprintln!("runtime error: {}", e.message),
    }
}

fn history_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("VELA_HISTORY") {
        return Some(PathBuf::from(p));
    }
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local").join("state"))
        })?;
    let dir = base.join("vela");
    let _ = fs::create_dir_all(&dir);
    Some(dir.join("history"))
}
