//! Read-eval-print loop for the Vela language.

use rustyline::completion::{Completer, Pair};
use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::{ValidationContext, ValidationResult, Validator};
use rustyline::{Cmd, Context, Editor, EventHandler, Helper, KeyCode, KeyEvent, Modifiers};
use std::borrow::Cow;
use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Instant;

pub fn run() -> i32 {
    let sess = Rc::new(RefCell::new(Session::new()));
    let interactive = std::io::IsTerminal::is_terminal(&std::io::stdin());
    if !interactive {
        return run_piped(&sess);
    }
    let helper = VelaHelper::new(Rc::clone(&sess));
    let editor: rustyline::Result<Editor<VelaHelper, rustyline::history::DefaultHistory>> =
        Editor::new();
    let mut ed = match editor {
        Ok(e) => e,
        Err(e) => {
            eprintln!("warning: line editor unavailable ({e}); falling back to plain input");
            return run_piped(&sess);
        }
    };
    ed.set_helper(Some(helper));
    ed.set_auto_add_history(true);
    ed.bind_sequence(
        KeyEvent(KeyCode::Tab, Modifiers::NONE),
        EventHandler::from(Cmd::CompleteHint),
    );
    let hist = history_path();
    if let Some(path) = hist.as_ref() {
        let _ = ed.load_history(path);
    }
    let exit = run_interactive(&sess, &mut ed);
    if let Some(path) = hist.as_ref() {
        let _ = ed.save_history(path);
    }
    exit
}

fn run_interactive(
    sess: &Rc<RefCell<Session>>,
    ed: &mut Editor<VelaHelper, rustyline::history::DefaultHistory>,
) -> i32 {
    print_banner();
    loop {
        let res = ed.readline("vela> ");
        match res {
            Ok(input) => {
                let trimmed = input.trim_end();
                if trimmed.is_empty() {
                    continue;
                }
                if !input.contains('\n') {
                    if let Some(meta) = parse_meta(trimmed) {
                        let cont = run_meta(&mut sess.borrow_mut(), meta);
                        if !cont {
                            return 0;
                        }
                        continue;
                    }
                }
                run_chunk(&mut sess.borrow_mut(), &input);
            }
            Err(ReadlineError::Interrupted) => {
                println!("(canceled)");
            }
            Err(ReadlineError::Eof) => return 0,
            Err(e) => {
                eprintln!("readline error: {e}");
                return 1;
            }
        }
    }
}

fn run_piped(sess: &Rc<RefCell<Session>>) -> i32 {
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
                if !run_meta(&mut sess.borrow_mut(), meta) {
                    return 0;
                }
                continue;
            }
        }
        if line.trim().is_empty() {
            if !buf.trim().is_empty() {
                let chunk = std::mem::take(&mut buf);
                run_chunk(&mut sess.borrow_mut(), &chunk);
            }
            continue;
        }
        if !buf.is_empty() {
            buf.push('\n');
        }
        buf.push_str(&line);
        if input_complete(&buf) {
            let chunk = std::mem::take(&mut buf);
            run_chunk(&mut sess.borrow_mut(), &chunk);
        }
    }
    if !buf.trim().is_empty() {
        let chunk = std::mem::take(&mut buf);
        run_chunk(&mut sess.borrow_mut(), &chunk);
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

    pub fn names(&self) -> Vec<String> {
        self.check.names()
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
    Names,
    Time(&'a str),
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
        "names" | "env" => Some(Meta::Names),
        "time" => Some(Meta::Time(rest)),
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
        Meta::Names => {
            for n in sess.names() {
                println!("{n}");
            }
            true
        }
        Meta::Time(src) => {
            if src.is_empty() {
                eprintln!("usage: :time EXPR");
            } else {
                run_chunk_timed(sess, src, true);
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
    println!("  :names | :env        list current bindings");
    println!("  :time EXPR           run EXPR and print elapsed time");
    println!();
    println!("Editing:");
    println!("  Enter                submit if input is complete, otherwise continue");
    println!("  blank line           force-submit the current buffer");
    println!("  Tab                  complete keyword, binding, or :command");
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
    run_chunk_timed(sess, src, false);
}

fn run_chunk_timed(sess: &mut Session, src: &str, show_time: bool) {
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
    let start = Instant::now();
    let result = sess.eval.eval_str(src);
    let elapsed = start.elapsed();
    match result {
        Ok(v) => match v {
            vela_eval::Value::Unit => {}
            other => println!("{} : {}", pretty(&other), ty.display()),
        },
        Err(e) => eprintln!("runtime error: {}", e.message),
    }
    if show_time {
        println!("(time {:.3?})", elapsed);
    }
}

fn pretty(v: &vela_eval::Value) -> String {
    let raw = vela_eval::show(v);
    if raw.len() <= 80 || !raw.contains(", ") {
        return raw;
    }
    pretty_multiline(v, 0)
}

fn pretty_multiline(v: &vela_eval::Value, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    let inner = "  ".repeat(depth + 1);
    match v {
        vela_eval::Value::Series(xs) if !xs.is_empty() => {
            let parts: Vec<String> = xs.iter().map(|x| pretty_multiline(x, depth + 1)).collect();
            format!(
                "[\n{inner}{}\n{indent}]",
                parts.join(&format!(",\n{inner}"))
            )
        }
        vela_eval::Value::Tuple(xs) if !xs.is_empty() => {
            let parts: Vec<String> = xs.iter().map(|x| pretty_multiline(x, depth + 1)).collect();
            format!(
                "(\n{inner}{}\n{indent})",
                parts.join(&format!(",\n{inner}"))
            )
        }
        vela_eval::Value::Record(fs) if !fs.is_empty() => {
            let parts: Vec<String> = fs
                .iter()
                .map(|(n, x)| format!("{n} = {}", pretty_multiline(x, depth + 1)))
                .collect();
            format!(
                "{{\n{inner}{}\n{indent}}}",
                parts.join(&format!(",\n{inner}"))
            )
        }
        other => vela_eval::show(other),
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

const KEYWORDS: &[&str] = &[
    "let", "rec", "var", "fn", "if", "then", "else", "match", "with", "when", "type", "trait",
    "impl", "for", "in", "pub", "module", "import", "as", "where", "scope", "spawn", "extern",
    "open", "app", "input", "output", "tests", "test", "prop", "true", "false", "and", "or", "not",
    "Some", "None", "Ok", "Err",
];

const META_CMDS: &[&str] = &[
    ":help", ":quit", ":exit", ":reset", ":type", ":load", ":names", ":env",
];

struct VelaHelper {
    sess: Rc<RefCell<Session>>,
    hinter: HistoryHinter,
}

impl VelaHelper {
    fn new(sess: Rc<RefCell<Session>>) -> Self {
        Self {
            sess,
            hinter: HistoryHinter::new(),
        }
    }
}

impl Helper for VelaHelper {}

impl Completer for VelaHelper {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let prefix_start = ident_start(&line[..pos]);
        let word = &line[prefix_start..pos];
        let mut candidates: Vec<Pair> = Vec::new();
        let starts_line = line[..prefix_start].chars().all(|c| c.is_whitespace());
        if starts_line && word.starts_with(':') {
            for cmd in META_CMDS {
                if cmd.starts_with(word) {
                    candidates.push(Pair {
                        display: cmd.to_string(),
                        replacement: cmd.to_string(),
                    });
                }
            }
            return Ok((prefix_start, candidates));
        }
        if word.is_empty() {
            return Ok((prefix_start, candidates));
        }
        for kw in KEYWORDS {
            if kw.starts_with(word) {
                candidates.push(Pair {
                    display: kw.to_string(),
                    replacement: kw.to_string(),
                });
            }
        }
        for name in self.sess.borrow().names() {
            if name.starts_with(word) && !KEYWORDS.contains(&name.as_str()) {
                candidates.push(Pair {
                    display: name.clone(),
                    replacement: name,
                });
            }
        }
        candidates.sort_by(|a, b| a.display.cmp(&b.display));
        candidates.dedup_by(|a, b| a.display == b.display);
        Ok((prefix_start, candidates))
    }
}

impl Hinter for VelaHelper {
    type Hint = String;
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for VelaHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Cow::Owned(colorize(line))
    }
    fn highlight_char(&self, _line: &str, _pos: usize, kind: CmdKind) -> bool {
        matches!(kind, CmdKind::ForcedRefresh | CmdKind::Other)
    }
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Cow::Owned(format!("\x1b[90m{hint}\x1b[0m"))
    }
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Cow::Owned(format!("\x1b[1;34m{prompt}\x1b[0m"))
        } else {
            Cow::Borrowed(prompt)
        }
    }
}

fn colorize(line: &str) -> String {
    let mut out = String::with_capacity(line.len() + 16);
    let bytes = line.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'#' {
            out.push_str("\x1b[90m");
            while i < bytes.len() && bytes[i] != b'\n' {
                out.push(bytes[i] as char);
                i += 1;
            }
            out.push_str("\x1b[0m");
            continue;
        }
        if b == b'"' {
            out.push_str("\x1b[32m");
            out.push('"');
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' && i + 1 < bytes.len() {
                    out.push('\\');
                    out.push(bytes[i + 1] as char);
                    i += 2;
                } else {
                    out.push(bytes[i] as char);
                    i += 1;
                }
            }
            if i < bytes.len() {
                out.push('"');
                i += 1;
            }
            out.push_str("\x1b[0m");
            continue;
        }
        if b.is_ascii_digit() {
            out.push_str("\x1b[33m");
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'.' || bytes[i] == b'_')
            {
                out.push(bytes[i] as char);
                i += 1;
            }
            out.push_str("\x1b[0m");
            continue;
        }
        if b == b':' && i + 1 < bytes.len() && bytes[i + 1].is_ascii_alphabetic() {
            let line_start = line[..i]
                .chars()
                .rev()
                .take_while(|c| *c != '\n')
                .all(|c| c.is_whitespace());
            if line_start {
                out.push_str("\x1b[36m");
                while i < bytes.len() && (bytes[i].is_ascii_alphabetic() || bytes[i] == b':') {
                    out.push(bytes[i] as char);
                    i += 1;
                }
                out.push_str("\x1b[0m");
                continue;
            }
        }
        if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            let word = &line[start..i];
            if is_keyword(word) {
                out.push_str("\x1b[1;35m");
                out.push_str(word);
                out.push_str("\x1b[0m");
            } else if word.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
                out.push_str("\x1b[36m");
                out.push_str(word);
                out.push_str("\x1b[0m");
            } else {
                out.push_str(word);
            }
            continue;
        }
        out.push(b as char);
        i += 1;
    }
    out
}

fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "let"
            | "rec"
            | "var"
            | "fn"
            | "if"
            | "then"
            | "else"
            | "match"
            | "with"
            | "when"
            | "type"
            | "trait"
            | "impl"
            | "for"
            | "in"
            | "pub"
            | "module"
            | "import"
            | "as"
            | "where"
            | "scope"
            | "spawn"
            | "extern"
            | "open"
            | "app"
            | "input"
            | "output"
            | "tests"
            | "test"
            | "prop"
            | "and"
            | "or"
            | "not"
            | "true"
            | "false"
    )
}

impl Validator for VelaHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let input = ctx.input();
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Ok(ValidationResult::Valid(None));
        }
        if trimmed.starts_with(':') && !input.contains('\n') {
            return Ok(ValidationResult::Valid(None));
        }
        if input_complete(input) {
            Ok(ValidationResult::Valid(None))
        } else {
            Ok(ValidationResult::Incomplete)
        }
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}

fn ident_start(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = bytes.len();
    while i > 0 {
        let c = bytes[i - 1];
        if c == b':' {
            return i - 1;
        }
        if c.is_ascii_alphanumeric() || c == b'_' {
            i -= 1;
        } else {
            break;
        }
    }
    i
}
