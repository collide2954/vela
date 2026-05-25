//! Source-span-aware diagnostic rendering for the Vela toolchain.

use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

impl Severity {
    fn label(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Option<String>,
    pub message: String,
    pub primary: Option<Range<usize>>,
    pub source_path: String,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code: None,
            message: message.into(),
            primary: None,
            source_path: "<unknown>".into(),
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn with_span(mut self, span: Range<usize>) -> Self {
        self.primary = Some(span);
        self
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.source_path = path.into();
        self
    }

    pub fn render(&self, source: &str) -> String {
        let mut out = String::new();
        let header = match &self.code {
            Some(c) => format!("{}[{c}]: {}", self.severity.label(), self.message),
            None => format!("{}: {}", self.severity.label(), self.message),
        };
        out.push_str(&header);
        out.push('\n');
        if let Some(span) = &self.primary {
            let (line_no, col_no, line_text) = line_info(source, span.start);
            out.push_str(&format!(
                "  --> {}:{line_no}:{col_no}\n",
                self.source_path
            ));
            let gutter_width = digit_count(line_no).max(2);
            let gutter = " ".repeat(gutter_width);
            out.push_str(&format!("{gutter} |\n"));
            out.push_str(&format!("{:>w$} | {line_text}\n", line_no, w = gutter_width));
            let span_in_line_start = col_no.saturating_sub(1);
            let span_in_line_len =
                span.len().min(line_text.len().saturating_sub(span_in_line_start)).max(1);
            let carets = "^".repeat(span_in_line_len);
            let pad = " ".repeat(span_in_line_start);
            out.push_str(&format!("{gutter} | {pad}{carets}\n"));
        }
        out
    }
}

fn line_info(source: &str, offset: usize) -> (usize, usize, &str) {
    let bytes = source.as_bytes();
    let clamped = offset.min(bytes.len());
    let mut line_start = 0usize;
    let mut line_no = 1usize;
    for (i, &b) in bytes.iter().enumerate().take(clamped) {
        if b == b'\n' {
            line_start = i + 1;
            line_no += 1;
        }
    }
    let line_end = bytes[line_start..]
        .iter()
        .position(|&b| b == b'\n')
        .map(|p| line_start + p)
        .unwrap_or(bytes.len());
    let line_text = &source[line_start..line_end];
    let col_no = clamped - line_start + 1;
    (line_no, col_no, line_text)
}

fn digit_count(mut n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    let mut d = 0;
    while n > 0 {
        d += 1;
        n /= 10;
    }
    d
}
