//! Lexical analysis for the Vela language.

use std::ops::Range;

pub type Span = Range<usize>;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Int(i64),
    UInt(u64),
    BigInt(String),
    Float(f64),
    Decimal(String),
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

    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.src.as_bytes().get(self.pos + offset).copied()
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

        if self.peek() == Some(b'0') && matches!(self.peek_at(1), Some(b'x' | b'X')) {
            return self.lex_radix(start, 16);
        }
        if self.peek() == Some(b'0') && matches!(self.peek_at(1), Some(b'b' | b'B')) {
            return self.lex_radix(start, 2);
        }

        let mut buf = String::new();
        let mut is_float = false;

        self.eat_digits(&mut buf);

        if self.peek() == Some(b'.') && self.peek_at(1).is_some_and(|b| b.is_ascii_digit()) {
            is_float = true;
            buf.push('.');
            self.pos += 1;
            self.eat_digits(&mut buf);
        }

        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            buf.push('e');
            self.pos += 1;
            if let Some(sign @ (b'+' | b'-')) = self.peek() {
                buf.push(sign as char);
                self.pos += 1;
            }
            self.eat_digits(&mut buf);
        }

        let kind = match self.suffix() {
            Some(b'u') if !is_float => {
                self.pos += 1;
                TokenKind::UInt(buf.parse().expect("digit-only u64 parses"))
            }
            Some(b'n') if !is_float => {
                self.pos += 1;
                TokenKind::BigInt(buf)
            }
            Some(b'd') => {
                self.pos += 1;
                TokenKind::Decimal(buf)
            }
            _ if is_float => {
                TokenKind::Float(buf.parse().expect("digit-only float buffer parses"))
            }
            _ => TokenKind::Int(buf.parse().expect("digit-only int buffer parses")),
        };

        Token { kind, span: start..self.pos }
    }

    fn suffix(&self) -> Option<u8> {
        let s = self.peek()?;
        if matches!(s, b'u' | b'n' | b'd')
            && !self.peek_at(1).is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            Some(s)
        } else {
            None
        }
    }

    fn lex_radix(&mut self, start: usize, radix: u32) -> Token {
        self.pos += 2;
        let mut value: u64 = 0;
        while let Some(b) = self.peek() {
            let d = match b {
                b'0'..=b'9' => (b - b'0') as u32,
                b'a'..=b'f' => (b - b'a' + 10) as u32,
                b'A'..=b'F' => (b - b'A' + 10) as u32,
                b'_' => {
                    self.pos += 1;
                    continue;
                }
                _ => break,
            };
            if d >= radix {
                break;
            }
            value = value * u64::from(radix) + u64::from(d);
            self.pos += 1;
        }
        let kind = match self.suffix() {
            Some(b'u') => {
                self.pos += 1;
                TokenKind::UInt(value)
            }
            Some(b'n') => {
                self.pos += 1;
                TokenKind::BigInt(value.to_string())
            }
            _ => TokenKind::Int(value as i64),
        };
        Token { kind, span: start..self.pos }
    }

    fn eat_digits(&mut self, buf: &mut String) {
        while let Some(b) = self.peek() {
            match b {
                b'0'..=b'9' => {
                    buf.push(b as char);
                    self.pos += 1;
                }
                b'_' => {
                    self.pos += 1;
                }
                _ => break,
            }
        }
    }

    fn lex_word(&mut self) -> Option<Token> {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b.is_ascii_alphanumeric() || b == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let text = &self.src[start..self.pos];
        let kind = match text {
            "NaN" => TokenKind::Float(f64::NAN),
            "Inf" => TokenKind::Float(f64::INFINITY),
            _ => return None,
        };
        Some(Token { kind, span: start..self.pos })
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let b = self.peek()?;
        if b.is_ascii_digit() {
            Some(self.lex_number())
        } else if b.is_ascii_alphabetic() || b == b'_' {
            self.lex_word()
        } else {
            None
        }
    }
}
