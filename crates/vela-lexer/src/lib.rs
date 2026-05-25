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
    Str(String),
    Bool(bool),
    Ident(String),
    Sym(String),
    Keyword(Keyword),
    Op(Op),
    Punct(Punct),
    DocComment(String),
    ModDoc(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Dot,
    Question,
    Caret,
    Star,
    Slash,
    Percent,
    Plus,
    Minus,
    PlusPlus,
    Eq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    Tilde,
    Pipe,
    Assign,
    LArrow,
    RArrow,
    DotDot,
    DotDotEq,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Punct {
    Colon,
    Comma,
    Semi,
    Bar,
    Tick,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    ArrayOpen,
    ArrayClose,
    FrameOpen,
    FrameClose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Let,
    Var,
    Fn,
    If,
    Then,
    Else,
    Match,
    With,
    When,
    Type,
    Trait,
    Impl,
    For,
    In,
    Return,
    Pub,
    Module,
    Import,
    As,
    Where,
    Scope,
    Spawn,
    Extern,
    Open,
    App,
    Input,
    Output,
    Tests,
    Test,
    Prop,
    And,
    Or,
    Not,
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
        loop {
            while let Some(b) = self.peek() {
                if b == b' ' || b == b'\t' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if self.peek() == Some(b'#') {
                self.skip_to_eol();
                continue;
            }
            if self.peek() == Some(b'/')
                && self.peek_at(1) == Some(b'/')
                && !matches!(self.peek_at(2), Some(b'/') | Some(b'!'))
            {
                self.skip_to_eol();
                continue;
            }
            break;
        }
    }

    fn skip_to_eol(&mut self) {
        while let Some(b) = self.peek() {
            self.pos += 1;
            if b == b'\n' {
                break;
            }
        }
    }

    fn read_to_eol(&mut self) -> String {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b == b'\n' {
                break;
            }
            self.pos += 1;
        }
        let text = self.src[start..self.pos].trim().to_string();
        if self.peek() == Some(b'\n') {
            self.pos += 1;
        }
        text
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

    fn lex_symbol(&mut self) -> Token {
        let start = self.pos;
        self.pos += 1;
        let body_start = self.pos;
        while let Some(b) = self.peek() {
            if b.is_ascii_alphanumeric() || b == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        let body = self.src[body_start..self.pos].to_string();
        Token { kind: TokenKind::Sym(body), span: start..self.pos }
    }

    fn lex_string(&mut self) -> Token {
        let start = self.pos;
        self.pos += 1;
        let mut buf = String::new();
        while let Some(b) = self.peek() {
            match b {
                b'"' => {
                    self.pos += 1;
                    return Token { kind: TokenKind::Str(buf), span: start..self.pos };
                }
                b'\\' => {
                    self.pos += 1;
                    let next = self.peek().unwrap_or(b'\\');
                    let ch = match next {
                        b'n' => '\n',
                        b't' => '\t',
                        b'r' => '\r',
                        b'"' => '"',
                        b'\\' => '\\',
                        b'0' => '\0',
                        other => other as char,
                    };
                    buf.push(ch);
                    self.pos += 1;
                }
                _ => {
                    buf.push(b as char);
                    self.pos += 1;
                }
            }
        }
        Token { kind: TokenKind::Str(buf), span: start..self.pos }
    }

    fn lex_word(&mut self) -> Token {
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
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            "let" => TokenKind::Keyword(Keyword::Let),
            "var" => TokenKind::Keyword(Keyword::Var),
            "fn" => TokenKind::Keyword(Keyword::Fn),
            "if" => TokenKind::Keyword(Keyword::If),
            "then" => TokenKind::Keyword(Keyword::Then),
            "else" => TokenKind::Keyword(Keyword::Else),
            "match" => TokenKind::Keyword(Keyword::Match),
            "with" => TokenKind::Keyword(Keyword::With),
            "when" => TokenKind::Keyword(Keyword::When),
            "type" => TokenKind::Keyword(Keyword::Type),
            "trait" => TokenKind::Keyword(Keyword::Trait),
            "impl" => TokenKind::Keyword(Keyword::Impl),
            "for" => TokenKind::Keyword(Keyword::For),
            "in" => TokenKind::Keyword(Keyword::In),
            "return" => TokenKind::Keyword(Keyword::Return),
            "pub" => TokenKind::Keyword(Keyword::Pub),
            "module" => TokenKind::Keyword(Keyword::Module),
            "import" => TokenKind::Keyword(Keyword::Import),
            "as" => TokenKind::Keyword(Keyword::As),
            "where" => TokenKind::Keyword(Keyword::Where),
            "scope" => TokenKind::Keyword(Keyword::Scope),
            "spawn" => TokenKind::Keyword(Keyword::Spawn),
            "extern" => TokenKind::Keyword(Keyword::Extern),
            "open" => TokenKind::Keyword(Keyword::Open),
            "app" => TokenKind::Keyword(Keyword::App),
            "input" => TokenKind::Keyword(Keyword::Input),
            "output" => TokenKind::Keyword(Keyword::Output),
            "tests" => TokenKind::Keyword(Keyword::Tests),
            "test" => TokenKind::Keyword(Keyword::Test),
            "prop" => TokenKind::Keyword(Keyword::Prop),
            "and" => TokenKind::Keyword(Keyword::And),
            "or" => TokenKind::Keyword(Keyword::Or),
            "not" => TokenKind::Keyword(Keyword::Not),
            other => TokenKind::Ident(other.to_string()),
        };
        Token { kind, span: start..self.pos }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.skip_whitespace();
        let b = self.peek()?;
        if b == b'/' && self.peek_at(1) == Some(b'/') {
            let start = self.pos;
            return match self.peek_at(2) {
                Some(b'/') => {
                    self.pos += 3;
                    let body = self.read_to_eol();
                    Some(Token { kind: TokenKind::DocComment(body), span: start..self.pos })
                }
                Some(b'!') => {
                    self.pos += 3;
                    let body = self.read_to_eol();
                    Some(Token { kind: TokenKind::ModDoc(body), span: start..self.pos })
                }
                _ => unreachable!("// without doc form was skipped by skip_whitespace"),
            };
        }
        if b.is_ascii_digit() {
            return Some(self.lex_number());
        }
        if b == b'"' {
            return Some(self.lex_string());
        }
        if b == b':' && self.peek_at(1).is_some_and(|b| b.is_ascii_alphabetic() || b == b'_') {
            return Some(self.lex_symbol());
        }
        if b.is_ascii_alphabetic() || b == b'_' {
            return Some(self.lex_word());
        }
        self.lex_punct()
    }
}

impl Lexer<'_> {
    fn lex_punct(&mut self) -> Option<Token> {
        let start = self.pos;
        let b = self.peek()?;
        let mut kind = None;
        match b {
            b'(' => kind = Some(TokenKind::Punct(Punct::LParen)),
            b')' => kind = Some(TokenKind::Punct(Punct::RParen)),
            b']' => kind = Some(TokenKind::Punct(Punct::RBracket)),
            b'}' => kind = Some(TokenKind::Punct(Punct::RBrace)),
            b',' => kind = Some(TokenKind::Punct(Punct::Comma)),
            b';' => kind = Some(TokenKind::Punct(Punct::Semi)),
            b':' => kind = Some(TokenKind::Punct(Punct::Colon)),
            b'\'' => kind = Some(TokenKind::Punct(Punct::Tick)),
            b'?' => kind = Some(TokenKind::Op(Op::Question)),
            b'^' => kind = Some(TokenKind::Op(Op::Caret)),
            b'*' => kind = Some(TokenKind::Op(Op::Star)),
            b'/' => kind = Some(TokenKind::Op(Op::Slash)),
            b'%' => kind = Some(TokenKind::Op(Op::Percent)),
            b'~' => kind = Some(TokenKind::Op(Op::Tilde)),
            b'[' => {
                if self.peek_at(1) == Some(b'|') {
                    self.pos += 2;
                    return Some(Token {
                        kind: TokenKind::Punct(Punct::ArrayOpen),
                        span: start..self.pos,
                    });
                }
                kind = Some(TokenKind::Punct(Punct::LBracket));
            }
            b'{' => {
                if self.peek_at(1) == Some(b'|') {
                    self.pos += 2;
                    return Some(Token {
                        kind: TokenKind::Punct(Punct::FrameOpen),
                        span: start..self.pos,
                    });
                }
                kind = Some(TokenKind::Punct(Punct::LBrace));
            }
            b'+' => {
                if self.peek_at(1) == Some(b'+') {
                    self.pos += 2;
                    return Some(Token {
                        kind: TokenKind::Op(Op::PlusPlus),
                        span: start..self.pos,
                    });
                }
                kind = Some(TokenKind::Op(Op::Plus));
            }
            b'-' => {
                if self.peek_at(1) == Some(b'>') {
                    self.pos += 2;
                    return Some(Token {
                        kind: TokenKind::Op(Op::RArrow),
                        span: start..self.pos,
                    });
                }
                kind = Some(TokenKind::Op(Op::Minus));
            }
            b'=' => {
                if self.peek_at(1) == Some(b'=') {
                    self.pos += 2;
                    return Some(Token { kind: TokenKind::Op(Op::Eq), span: start..self.pos });
                }
                kind = Some(TokenKind::Op(Op::Assign));
            }
            b'!' => {
                if self.peek_at(1) == Some(b'=') {
                    self.pos += 2;
                    return Some(Token {
                        kind: TokenKind::Op(Op::NotEq),
                        span: start..self.pos,
                    });
                }
            }
            b'<' => {
                match self.peek_at(1) {
                    Some(b'-') => {
                        self.pos += 2;
                        return Some(Token {
                            kind: TokenKind::Op(Op::LArrow),
                            span: start..self.pos,
                        });
                    }
                    Some(b'=') => {
                        self.pos += 2;
                        return Some(Token {
                            kind: TokenKind::Op(Op::Le),
                            span: start..self.pos,
                        });
                    }
                    _ => kind = Some(TokenKind::Op(Op::Lt)),
                }
            }
            b'>' => {
                if self.peek_at(1) == Some(b'=') {
                    self.pos += 2;
                    return Some(Token { kind: TokenKind::Op(Op::Ge), span: start..self.pos });
                }
                kind = Some(TokenKind::Op(Op::Gt));
            }
            b'.' => {
                if self.peek_at(1) == Some(b'.') {
                    if self.peek_at(2) == Some(b'=') {
                        self.pos += 3;
                        return Some(Token {
                            kind: TokenKind::Op(Op::DotDotEq),
                            span: start..self.pos,
                        });
                    }
                    self.pos += 2;
                    return Some(Token {
                        kind: TokenKind::Op(Op::DotDot),
                        span: start..self.pos,
                    });
                }
                kind = Some(TokenKind::Op(Op::Dot));
            }
            b'|' => match self.peek_at(1) {
                Some(b'>') => {
                    self.pos += 2;
                    return Some(Token {
                        kind: TokenKind::Op(Op::Pipe),
                        span: start..self.pos,
                    });
                }
                Some(b']') => {
                    self.pos += 2;
                    return Some(Token {
                        kind: TokenKind::Punct(Punct::ArrayClose),
                        span: start..self.pos,
                    });
                }
                Some(b'}') => {
                    self.pos += 2;
                    return Some(Token {
                        kind: TokenKind::Punct(Punct::FrameClose),
                        span: start..self.pos,
                    });
                }
                _ => kind = Some(TokenKind::Punct(Punct::Bar)),
            },
            _ => return None,
        }
        self.pos += 1;
        kind.map(|k| Token { kind: k, span: start..self.pos })
    }
}
