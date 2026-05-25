//! Lexical analysis for the Vela language.

use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Int(i64),
}

pub fn lex(src: &str) -> Lexer<'_> {
    Lexer::new(src)
}

pub struct Lexer<'a> {
    src: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(src: &'a str) -> Self {
        Self { src, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.src.as_bytes().get(self.pos).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(b) = self.peek() {
            if b == b' ' || b == b'\t' {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn lex_number(&mut self) -> Token {
        let start = self.pos;
        let mut value: i64 = 0;
        while let Some(b) = self.peek() {
            match b {
                b'0'..=b'9' => {
                    value = value * 10 + (b - b'0') as i64;
                    self.pos += 1;
                }
                b'_' => {
                    self.pos += 1;
                }
                _ => break,
            }
        }
        Token { kind: TokenKind::Int(value), span: start..self.pos }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let b = self.peek()?;
        if b.is_ascii_digit() {
            Some(self.lex_number())
        } else {
            None
        }
    }
}
