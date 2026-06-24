//! [50200] Parseur de bindings `.iri` (interop niveau 2).
//!
//! Format déclaratif : `extern fn name(p: T, …) -> R = "rust::path"`, `extern type Name = "rust::Path"`,
//! `crate name = "version"`. Réutilise le lexer Iron.

use crate::ir::*;
use crate::lex;

pub fn parse(src: &str, file: FileId) -> CResult<Bindings> {
    let toks = lex::lex(src, file)?;
    P { t: toks, i: 0 }.run()
}

struct P {
    t: Vec<Token>,
    i: usize,
}

impl P {
    fn cur(&self) -> &TokenKind {
        &self.t[self.i].kind
    }
    fn adv(&mut self) {
        if self.i + 1 < self.t.len() {
            self.i += 1;
        }
    }
    fn is(&self, k: &TokenKind) -> bool {
        std::mem::discriminant(self.cur()) == std::mem::discriminant(k)
    }
    fn eat(&mut self, k: &TokenKind) -> bool {
        if self.is(k) {
            self.adv();
            true
        } else {
            false
        }
    }
    fn kw(&self, w: &str) -> bool {
        matches!(self.cur(), TokenKind::Ident(s) if s == w)
    }
    fn err(&self, m: impl Into<String>) -> CompileError {
        CompileError { span: Some(self.t[self.i].span), msg: m.into() }
    }
    fn expect(&mut self, k: &TokenKind, w: &str) -> CResult<()> {
        if self.eat(k) {
            Ok(())
        } else {
            Err(self.err(format!("attendu {w}, trouvé {:?}", self.cur())))
        }
    }
    fn ident(&mut self) -> CResult<String> {
        if let TokenKind::Ident(s) = self.cur() {
            let s = s.clone();
            self.adv();
            Ok(s)
        } else {
            Err(self.err("identifiant attendu"))
        }
    }
    fn string(&mut self) -> CResult<String> {
        if let TokenKind::Str(s) = self.cur() {
            let s = s.clone();
            self.adv();
            Ok(s)
        } else {
            Err(self.err("chaîne (chemin Rust) attendue"))
        }
    }
    fn skip_nl(&mut self) {
        while self.is(&TokenKind::Newline) {
            self.adv();
        }
    }

    fn run(mut self) -> CResult<Bindings> {
        let mut b = Bindings::default();
        loop {
            self.skip_nl();
            if self.is(&TokenKind::Eof) {
                break;
            }
            if self.kw("extern") {
                self.adv();
                if self.is(&TokenKind::Fn) {
                    self.adv();
                    let name = self.ident()?;
                    self.expect(&TokenKind::LParen, "`(`")?;
                    let mut params = Vec::new();
                    while !self.is(&TokenKind::RParen) {
                        let _pn = self.ident()?;
                        self.expect(&TokenKind::Colon, "`:`")?;
                        params.push(self.parse_type()?);
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RParen, "`)`")?;
                    let ret = if self.eat(&TokenKind::Arrow) {
                        self.parse_type()?
                    } else {
                        Type::Unit
                    };
                    self.expect(&TokenKind::Assign, "`=`")?;
                    let path = self.string()?;
                    b.fns.insert(name, ExternFn { params, ret, path });
                } else if self.kw("type") {
                    self.adv();
                    let name = self.ident()?;
                    self.expect(&TokenKind::Assign, "`=`")?;
                    let path = self.string()?;
                    b.types.insert(name, path);
                } else {
                    return Err(self.err("`extern fn` ou `extern type` attendu"));
                }
            } else if self.kw("crate") {
                self.adv();
                let name = self.ident()?;
                self.expect(&TokenKind::Assign, "`=`")?;
                let ver = self.string()?;
                b.crates.push((name, ver));
            } else {
                return Err(self.err("`extern` ou `crate` attendu"));
            }
            self.skip_nl();
        }
        // Résolution des types opaques : `Extern(nom Iron)` → `Extern(chemin Rust)`.
        let types = b.types.clone();
        for f in b.fns.values_mut() {
            for p in &mut f.params {
                resolve_extern(p, &types);
            }
            resolve_extern(&mut f.ret, &types);
        }
        Ok(b)
    }

    fn parse_type(&mut self) -> CResult<Type> {
        let name = self.ident()?;
        match name.as_str() {
            "i64" => Ok(Type::I64),
            "f64" => Ok(Type::F64),
            "bool" => Ok(Type::Bool),
            "str" => Ok(Type::Str),
            "unit" => Ok(Type::Unit),
            "list" => {
                self.expect(&TokenKind::LBracket, "`[`")?;
                let t = self.parse_type()?;
                self.expect(&TokenKind::RBracket, "`]`")?;
                Ok(Type::List(Box::new(t)))
            }
            "dict" => {
                self.expect(&TokenKind::LBracket, "`[`")?;
                let k = self.parse_type()?;
                self.expect(&TokenKind::Comma, "`,`")?;
                let v = self.parse_type()?;
                self.expect(&TokenKind::RBracket, "`]`")?;
                Ok(Type::Dict(Box::new(k), Box::new(v)))
            }
            "Option" => {
                self.expect(&TokenKind::LBracket, "`[`")?;
                let t = self.parse_type()?;
                self.expect(&TokenKind::RBracket, "`]`")?;
                Ok(Type::Option(Box::new(t)))
            }
            "Result" => {
                self.expect(&TokenKind::LBracket, "`[`")?;
                let t = self.parse_type()?;
                self.expect(&TokenKind::RBracket, "`]`")?;
                Ok(Type::Result(Box::new(t)))
            }
            // nom de type opaque (résolu en chemin Rust après coup)
            other => Ok(Type::Extern(other.to_string())),
        }
    }
}

fn resolve_extern(t: &mut Type, types: &std::collections::HashMap<String, String>) {
    match t {
        Type::Extern(name) => {
            if let Some(path) = types.get(name) {
                *name = path.clone();
            }
        }
        Type::List(e) | Type::Option(e) | Type::Result(e) => resolve_extern(e, types),
        Type::Dict(k, v) => {
            resolve_extern(k, types);
            resolve_extern(v, types);
        }
        _ => {}
    }
}
