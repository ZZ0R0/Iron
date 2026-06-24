//! [10100] Lexer indentation-sensitive — réel (planify-2.1).
//!
//! Off-side rule (Python) + suspension dans les régions `{}` (V0/README §1bis, §4.1).
//! Indentation = **espaces** ; tabulation/mélange = erreur lexicale. Spans = **octets**.

use crate::ir::*;

pub fn lex(src: &str, file: FileId) -> CResult<Vec<Token>> {
    Lexer::new(src.as_bytes(), file).run()
}

struct Lexer<'a> {
    src: &'a [u8],
    file: FileId,
    pos: usize,
    out: Vec<Token>,
    indents: Vec<usize>,
    brace_depth: usize,
    line_has_content: bool,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a [u8], file: FileId) -> Self {
        Lexer {
            src,
            file,
            pos: 0,
            out: Vec::new(),
            indents: vec![0],
            brace_depth: 0,
            line_has_content: false,
        }
    }

    fn err(&self, at: usize, msg: impl Into<String>) -> CompileError {
        CompileError {
            span: Some(Span::new(self.file, at as u32, (at + 1).min(self.src.len() + 1) as u32)),
            msg: msg.into(),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }
    fn peek2(&self) -> Option<u8> {
        self.src.get(self.pos + 1).copied()
    }
    fn span(&self, start: usize) -> Span {
        Span::new(self.file, start as u32, self.pos as u32)
    }
    fn at(&self, pos: usize) -> Span {
        Span::new(self.file, pos as u32, pos as u32)
    }

    fn push(&mut self, kind: TokenKind, start: usize) {
        let span = self.span(start);
        self.out.push(Token { kind, span });
        self.line_has_content = true;
    }

    fn run(mut self) -> CResult<Vec<Token>> {
        self.handle_line_start()?;
        while let Some(c) = self.peek() {
            match c {
                b'\n' => {
                    let start = self.pos;
                    self.pos += 1;
                    if self.line_has_content {
                        let span = Span::new(self.file, start as u32, self.pos as u32);
                        self.out.push(Token { kind: TokenKind::Newline, span });
                        self.line_has_content = false;
                    }
                    if self.brace_depth == 0 {
                        self.handle_line_start()?;
                    }
                }
                b' ' | b'\r' => self.pos += 1,
                b'\t' => return Err(self.err(self.pos, "tabulation interdite hors indentation")),
                b'#' => {
                    while let Some(c) = self.peek() {
                        if c == b'\n' {
                            break;
                        }
                        self.pos += 1;
                    }
                }
                b'0'..=b'9' => self.lex_number()?,
                c if is_ident_start(c) => self.lex_ident()?,
                b'"' => self.lex_string()?,
                _ => self.lex_op()?,
            }
        }
        if self.line_has_content {
            self.out.push(Token { kind: TokenKind::Newline, span: self.at(self.pos) });
        }
        while self.indents.len() > 1 {
            self.indents.pop();
            self.out.push(Token { kind: TokenKind::Dedent, span: self.at(self.pos) });
        }
        self.out.push(Token { kind: TokenKind::Eof, span: self.at(self.pos) });
        Ok(self.out)
    }

    /// Début de ligne logique (hors accolades) : mesure l'indentation (espaces),
    /// émet `Indent`/`Dedent`, saute lignes blanches et commentaires.
    fn handle_line_start(&mut self) -> CResult<()> {
        loop {
            let mut indent = 0usize;
            while let Some(c) = self.peek() {
                match c {
                    b' ' => {
                        indent += 1;
                        self.pos += 1;
                    }
                    b'\t' => return Err(self.err(self.pos, "indentation par tabulation interdite (espaces uniquement)")),
                    _ => break,
                }
            }
            match self.peek() {
                None => return Ok(()),
                Some(b'\n') => {
                    self.pos += 1;
                    continue;
                }
                Some(b'#') => {
                    while let Some(c) = self.peek() {
                        if c == b'\n' {
                            break;
                        }
                        self.pos += 1;
                    }
                    continue;
                }
                Some(_) => {
                    self.adjust_indent(indent);
                    return Ok(());
                }
            }
        }
    }

    fn adjust_indent(&mut self, indent: usize) {
        let cur = *self.indents.last().unwrap();
        if indent > cur {
            self.indents.push(indent);
            self.out.push(Token { kind: TokenKind::Indent, span: self.at(self.pos) });
        } else {
            while *self.indents.last().unwrap() > indent {
                self.indents.pop();
                self.out.push(Token { kind: TokenKind::Dedent, span: self.at(self.pos) });
            }
        }
    }

    fn lex_number(&mut self) -> CResult<()> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.pos += 1;
            } else {
                break;
            }
        }
        let mut is_float = false;
        if self.peek() == Some(b'.') && self.peek2().map_or(false, |c| c.is_ascii_digit()) {
            is_float = true;
            self.pos += 1;
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }
        let text = std::str::from_utf8(&self.src[start..self.pos]).unwrap();
        if is_float {
            let v: f64 = text.parse().map_err(|_| self.err(start, "littéral flottant invalide"))?;
            self.push(TokenKind::Float(v), start);
        } else {
            let v: i64 = text.parse().map_err(|_| self.err(start, "littéral entier trop grand"))?;
            self.push(TokenKind::Int(v), start);
        }
        Ok(())
    }

    fn lex_ident(&mut self) -> CResult<()> {
        let start = self.pos;
        while let Some(c) = self.peek() {
            if is_ident_continue(c) {
                self.pos += 1;
            } else {
                break;
            }
        }
        let text = std::str::from_utf8(&self.src[start..self.pos]).unwrap().to_string();
        // `rust {` → capture brute du bloc (interop niveau 1, 50100).
        if text == "rust" {
            let mut p = self.pos;
            while matches!(self.src.get(p), Some(b' ') | Some(b'\t')) {
                p += 1;
            }
            if self.src.get(p) == Some(&b'{') {
                self.pos = p + 1; // après le `{`
                let inner = self.capture_rust_block(start)?;
                self.push(TokenKind::RustVerbatim(inner), start);
                return Ok(());
            }
        }
        let kind = match text.as_str() {
            "fn" => TokenKind::Fn,
            "struct" => TokenKind::Struct,
            "enum" => TokenKind::Enum,
            "match" => TokenKind::Match,
            "trait" => TokenKind::Trait,
            "impl" => TokenKind::Impl,
            "if" => TokenKind::If,
            "elif" => TokenKind::Elif,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "for" => TokenKind::For,
            "in" => TokenKind::In,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "return" => TokenKind::Return,
            "pass" => TokenKind::Pass,
            "and" => TokenKind::And,
            "or" => TokenKind::Or,
            "not" => TokenKind::Not,
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ => TokenKind::Ident(text.clone()),
        };
        self.push(kind, start);
        Ok(())
    }

    /// Capte le contenu brut d'un `rust { … }` jusqu'au `}` apparié (en sautant chaînes et `//`).
    fn capture_rust_block(&mut self, start: usize) -> CResult<String> {
        let inner_start = self.pos;
        let mut depth = 1usize;
        while depth > 0 {
            match self.peek() {
                None => return Err(self.err(start, "bloc `rust { }` non terminé")),
                Some(b'{') => {
                    depth += 1;
                    self.pos += 1;
                }
                Some(b'}') => {
                    depth -= 1;
                    self.pos += 1;
                }
                Some(b'"') => self.skip_rust_string()?,
                Some(b'/') if self.peek2() == Some(b'/') => {
                    while let Some(c) = self.peek() {
                        if c == b'\n' {
                            break;
                        }
                        self.pos += 1;
                    }
                }
                Some(_) => self.pos += 1,
            }
        }
        let inner = std::str::from_utf8(&self.src[inner_start..self.pos - 1])
            .unwrap_or("")
            .to_string();
        Ok(inner)
    }

    fn skip_rust_string(&mut self) -> CResult<()> {
        self.pos += 1; // guillemet ouvrant
        loop {
            match self.peek() {
                None => return Err(self.err(self.pos, "chaîne Rust non terminée dans `rust { }`")),
                Some(b'\\') => {
                    self.pos += 1;
                    if self.peek().is_some() {
                        self.pos += 1;
                    }
                }
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(());
                }
                Some(_) => self.pos += 1,
            }
        }
    }

    fn lex_string(&mut self) -> CResult<()> {
        let start = self.pos;
        self.pos += 1; // "
        let mut s = String::new();
        loop {
            match self.peek() {
                None => return Err(self.err(start, "chaîne non terminée")),
                Some(b'"') => {
                    self.pos += 1;
                    break;
                }
                Some(b'\\') => {
                    self.pos += 1;
                    match self.peek() {
                        Some(b'n') => s.push('\n'),
                        Some(b't') => s.push('\t'),
                        Some(b'"') => s.push('"'),
                        Some(b'\\') => s.push('\\'),
                        _ => return Err(self.err(self.pos, "échappement invalide")),
                    }
                    self.pos += 1;
                }
                Some(b'\n') => return Err(self.err(start, "chaîne non terminée")),
                Some(c) => {
                    s.push(c as char);
                    self.pos += 1;
                }
            }
        }
        self.push(TokenKind::Str(s), start);
        Ok(())
    }

    fn lex_op(&mut self) -> CResult<()> {
        let start = self.pos;
        let c = self.peek().unwrap();
        let two = self.peek2();
        macro_rules! one {
            ($k:expr) => {{
                self.pos += 1;
                self.push($k, start);
            }};
        }
        macro_rules! two_ {
            ($k:expr) => {{
                self.pos += 2;
                self.push($k, start);
            }};
        }
        match (c, two) {
            (b'=', Some(b'=')) => two_!(TokenKind::EqEq),
            (b'!', Some(b'=')) => two_!(TokenKind::NotEq),
            (b'<', Some(b'=')) => two_!(TokenKind::Le),
            (b'>', Some(b'=')) => two_!(TokenKind::Ge),
            (b'-', Some(b'>')) => two_!(TokenKind::Arrow),
            (b'=', _) => one!(TokenKind::Assign),
            (b'<', _) => one!(TokenKind::Lt),
            (b'>', _) => one!(TokenKind::Gt),
            (b'+', _) => one!(TokenKind::Plus),
            (b'-', _) => one!(TokenKind::Minus),
            (b'*', _) => one!(TokenKind::Star),
            (b'/', _) => one!(TokenKind::Slash),
            (b'%', _) => one!(TokenKind::Percent),
            (b'(', _) => one!(TokenKind::LParen),
            (b')', _) => one!(TokenKind::RParen),
            (b'[', _) => one!(TokenKind::LBracket),
            (b']', _) => one!(TokenKind::RBracket),
            (b';', _) => one!(TokenKind::Semicolon),
            (b',', _) => one!(TokenKind::Comma),
            (b'.', _) => one!(TokenKind::Dot),
            (b':', _) => one!(TokenKind::Colon),
            (b'?', _) => one!(TokenKind::Question),
            (b'{', _) => {
                self.brace_depth += 1;
                one!(TokenKind::LBrace);
            }
            (b'}', _) => {
                self.brace_depth = self.brace_depth.saturating_sub(1);
                one!(TokenKind::RBrace);
            }
            _ => return Err(self.err(start, format!("caractère illégal : {:?}", c as char))),
        }
        Ok(())
    }
}

fn is_ident_start(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphabetic()
}
fn is_ident_continue(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphanumeric()
}
