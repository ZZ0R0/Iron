//! [50200/50900] Parseur de bindings `.iri` (interop niveau 2, format v2 V8).
//!
//! Format déclaratif (rétro-compatible v1) :
//! - `crate name = "version"`
//! - `extern fn name(p: T, …) -> R = "rust::path" [idiomes]`
//! - `extern type Name = "rust::Path"`                     (opaque, sans API — 50400)
//! - `extern type Name = "rust::Path":` + bloc indenté de `fn …` (méthodes/ctors — 50900)
//! - `extern mod name:` + bloc indenté de noms d'items     (namespacing — 51500)
//!
//! Réceveur d'une méthode : 1ᵉʳ param `self` (`&self`/ByRef), `mut self` (`&mut self`/ByRefMut),
//! `own self` (`self`/ByValue). Sans `self` → constructeur (`recv = None`), aussi exposé en fn libre.
//! Idiomes : `[builder]`, `[clone]`, `[collect]`, `[closure]`.

use crate::ir::*;
use crate::lex;

pub fn parse(src: &str, file: FileId) -> CResult<Bindings> {
    let toks = lex::lex(src, file)?;
    P { t: toks, i: 0, cur_params: Vec::new() }.run()
}

struct P {
    t: Vec<Token>,
    i: usize,
    /// V12 [24000] : params de type du bloc `extern type Name[T]:` en cours — un nom de type qui matche
    /// est lu en `Type::Generic` (et non `Extern`), pour la substitution en inférence.
    cur_params: Vec<String>,
}

impl P {
    fn cur(&self) -> &TokenKind {
        &self.t[self.i].kind
    }
    fn peek(&self) -> &TokenKind {
        &self.t[(self.i + 1).min(self.t.len() - 1)].kind
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
    fn ident_is(&self, w: &str) -> bool {
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
                    let (name, f) = self.parse_fn(None)?;
                    b.fns.insert(name, f);
                } else if self.kw("type") {
                    self.adv();
                    self.parse_type_decl(&mut b)?;
                } else if self.kw("mod") {
                    self.adv();
                    self.parse_mod_decl(&mut b)?;
                } else if self.kw("const") {
                    // constante publique (V12 [50500]) : `extern const NOM: T = "chemin"`.
                    self.adv();
                    let name = self.ident()?;
                    self.expect(&TokenKind::Colon, "`:`")?;
                    let ty = self.parse_type()?;
                    self.expect(&TokenKind::Assign, "`=`")?;
                    let path = self.string()?;
                    b.consts.entry(name).or_insert((ty, path));
                } else if self.is(&TokenKind::Trait) {
                    // trait implémentable (V12 [24300]) : `extern trait T = "chemin"`. `trait` est un
                    // mot-clé Iron → token dédié (pas un `Ident`), d'où `is(Trait)` et non `kw("trait")`.
                    self.adv();
                    let name = self.ident()?;
                    self.expect(&TokenKind::Assign, "`=`")?;
                    let path = self.string()?;
                    b.traits.entry(name).or_insert(path);
                } else {
                    return Err(self.err("`extern fn`, `extern type`, `extern const`, `extern trait` ou `extern mod` attendu"));
                }
            } else if self.kw("crate") {
                self.adv();
                // nom de crate : peut contenir des tirets (`num-bigint`, `serde-json`) → on recolle
                // `ident (- ident)*`. C'est le nom **publié** (clé Cargo), distinct du nom d'import Iron
                // (underscore, = nom de fichier `.iri`).
                let mut name = self.ident()?;
                while self.eat(&TokenKind::Minus) {
                    name.push('-');
                    name.push_str(&self.ident()?);
                }
                self.expect(&TokenKind::Assign, "`=`")?;
                let ver = self.string()?;
                // features optionnelles (V9, 51900) : `crate serde = "1" features [derive]`.
                let mut feats = Vec::new();
                if self.kw("features") {
                    self.adv();
                    self.expect(&TokenKind::LBracket, "`[`")?;
                    while !self.is(&TokenKind::RBracket) {
                        feats.push(self.ident()?);
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RBracket, "`]`")?;
                }
                b.crates.push((name, ver, feats));
            } else {
                return Err(self.err("`extern` ou `crate` attendu"));
            }
            self.skip_nl();
        }
        // Résolution des types opaques sans API : `Extern(nom)` → `Extern(chemin Rust)` (rétro-compat 50400).
        // Les types à API (etypes) gardent leur **nom Iron** dans `Type::Extern` (infer résout les méthodes
        // par nom ; lower émet l'alias `type Nom = chemin;`).
        let opaque: std::collections::HashMap<String, String> =
            b.types.iter().filter(|(n, _)| !b.etypes.contains_key(*n)).map(|(n, p)| (n.clone(), p.clone())).collect();
        for f in b.fns.values_mut() {
            resolve_fn(f, &opaque);
        }
        let etypes: Vec<String> = b.etypes.keys().cloned().collect();
        for n in &etypes {
            let mut et = b.etypes.remove(n).unwrap();
            for (_, f) in et.ctors.iter_mut().chain(et.methods.iter_mut()) {
                resolve_fn(f, &opaque);
            }
            b.etypes.insert(n.clone(), et);
        }
        Ok(b)
    }

    /// `name(params) -> ret = "path" [idiomes]`. `owner` = chemin Rust du type si méthode (pour résoudre
    /// un chemin court). Renvoie `(name, ExternFn)` avec `recv` détecté.
    fn parse_fn(&mut self, owner: Option<&str>) -> CResult<(String, ExternFn)> {
        let name = self.ident()?;
        self.expect(&TokenKind::LParen, "`(`")?;
        let (recv, params, param_owned) = self.parse_params()?;
        self.expect(&TokenKind::RParen, "`)`")?;
        let ret = if self.eat(&TokenKind::Arrow) { self.parse_type()? } else { Type::Unit };
        self.expect(&TokenKind::Assign, "`=`")?;
        let mut path = self.string()?;
        // Préfixer par le chemin du type **uniquement pour un constructeur** (recv None) → `crate::Type::ctor`.
        // Une méthode garde son nom court (syntaxe-méthode, V9) ; une méthode de trait arrive déjà en chemin
        // complet `crate::Trait::method` (UFCS) depuis `bind`.
        if let Some(own) = owner {
            if recv.is_none() && !path.contains("::") {
                path = format!("{own}::{path}");
            }
        }
        let idioms = self.parse_idioms()?;
        Ok((name, ExternFn { params, param_owned, ret, path, recv, idioms }))
    }

    /// Réceveur (`self`/`mut self`/`own self`) puis params nommés. `param_owned[i]` = param `i` par valeur
    /// (`own T`).
    fn parse_params(&mut self) -> CResult<(Option<Recv>, Vec<Type>, Vec<bool>)> {
        let mut recv = None;
        if self.ident_is("self") {
            self.adv();
            recv = Some(Recv::ByRef);
        } else if self.ident_is("mut") && matches!(self.peek(), TokenKind::Ident(s) if s == "self") {
            self.adv();
            self.adv();
            recv = Some(Recv::ByRefMut);
        } else if self.ident_is("own") && matches!(self.peek(), TokenKind::Ident(s) if s == "self") {
            self.adv();
            self.adv();
            recv = Some(Recv::ByValue);
        }
        if recv.is_some() {
            self.eat(&TokenKind::Comma);
        }
        let mut params = Vec::new();
        let mut param_owned = Vec::new();
        while !self.is(&TokenKind::RParen) {
            let _pn = self.ident()?;
            self.expect(&TokenKind::Colon, "`:`")?;
            // `own T` → param par valeur (pas de `&` au lowering).
            let owned = self.ident_is("own");
            if owned {
                self.adv();
            }
            param_owned.push(owned);
            params.push(self.parse_type()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        Ok((recv, params, param_owned))
    }

    fn parse_idioms(&mut self) -> CResult<Vec<Idiom>> {
        let mut out = Vec::new();
        if !self.eat(&TokenKind::LBracket) {
            return Ok(out);
        }
        while !self.is(&TokenKind::RBracket) {
            let w = self.ident()?;
            out.push(match w.as_str() {
                "builder" => Idiom::Builder,
                "clone" => Idiom::Clone,
                "collect" => Idiom::Collect,
                "closure" => Idiom::Closure(0),
                "display" => Idiom::Display,
                other => return Err(self.err(format!("idiome inconnu `{other}` (builder/clone/collect/closure/display)"))),
            });
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::RBracket, "`]`")?;
        Ok(out)
    }

    fn parse_type_decl(&mut self, b: &mut Bindings) -> CResult<()> {
        let name = self.ident()?;
        // Paramètres de type : `extern type Point[T, U] = …` (V12, [24000]). Optionnel (rétro-compat).
        let mut params = Vec::new();
        if self.eat(&TokenKind::LBracket) {
            while !self.is(&TokenKind::RBracket) {
                params.push(self.ident()?);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::RBracket, "`]`")?;
        }
        self.expect(&TokenKind::Assign, "`=`")?;
        let path = self.string()?;
        // **first-wins** sur collision de nom court : le crate **principal** est émis avant les sources de
        // façade fusionnées (52700) ; on garde donc le type principal (`scraper::Selector` avec `parse`) au
        // lieu de le laisser écraser par un homonyme fusionné (`selectors::parser::Selector`). B31.
        b.types.entry(name.clone()).or_insert_with(|| path.clone());
        // Les noms de params sont lus en `Generic` dans le bloc (substitution infer).
        self.cur_params = params.clone();
        // Bloc indenté → type à API.
        if self.eat(&TokenKind::Colon) {
            let mut et = ExternType { rust_path: path.clone(), params: params.clone(), ..Default::default() };
            self.skip_nl();
            if self.eat(&TokenKind::Indent) {
                loop {
                    self.skip_nl();
                    if self.is(&TokenKind::Dedent) || self.is(&TokenKind::Eof) {
                        break;
                    }
                    if self.ident_is("field") {
                        // `field nom: type` (champ public d'une struct externe — construction par littéral).
                        self.adv();
                        let fname = self.ident()?;
                        self.expect(&TokenKind::Colon, "`:`")?;
                        let ty = self.parse_type()?;
                        et.fields.push((fname, ty));
                    } else if self.ident_is("case") {
                        // `case Variant` (variante d'un enum externe — `Enum.Variant`).
                        self.adv();
                        et.variants.push(self.ident()?);
                    } else if self.ident_is("base") {
                        // [50600 C1+] `base Nom` : type de base dont l'alias hérite les méthodes (Vector3 → Matrix).
                        self.adv();
                        et.base = Some(self.ident()?);
                    } else {
                        self.expect(&TokenKind::Fn, "`fn`/`field`/`case`")?;
                        let (fname, f) = self.parse_fn(Some(&path))?;
                        if f.recv.is_some() {
                            et.methods.push((fname, f));
                        } else {
                            // constructeur : aussi exposé en fn libre (appel `nom(...)`). **or_insert** (B34) :
                            // ne pas écraser une **vraie fn libre** de même nom (`extern fn`, insert) — un
                            // `from_str` serde de `toml` doit primer sur le ctor `Datetime::from_str`. Entre
                            // ctors homonymes, le 1ᵉʳ gagne (dédup stable) ; un `extern fn` ultérieur prime.
                            b.fns.entry(fname.clone()).or_insert_with(|| f.clone());
                            et.ctors.push((fname, f));
                        }
                    }
                    self.skip_nl();
                }
                self.eat(&TokenKind::Dedent);
            }
            // first-wins (B31) : ne pas laisser un homonyme de façade écraser le type du crate principal.
            b.etypes.entry(name).or_insert(et);
        }
        self.cur_params.clear();
        Ok(())
    }

    fn parse_mod_decl(&mut self, b: &mut Bindings) -> CResult<()> {
        let name = self.ident()?;
        self.expect(&TokenKind::Colon, "`:`")?;
        self.skip_nl();
        let mut items = Vec::new();
        if self.eat(&TokenKind::Indent) {
            loop {
                self.skip_nl();
                if self.is(&TokenKind::Dedent) || self.is(&TokenKind::Eof) {
                    break;
                }
                items.push(self.ident()?);
                self.skip_nl();
            }
            self.eat(&TokenKind::Dedent);
        }
        b.mods.insert(name, items);
        Ok(())
    }

    fn parse_type(&mut self) -> CResult<Type> {
        // tuple `(A, B, …)` ou unit `()` (V9.1 robustesse : bind produit ces types).
        if self.eat(&TokenKind::LParen) {
            let mut ts = Vec::new();
            while !self.is(&TokenKind::RParen) {
                ts.push(self.parse_type()?);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::RParen, "`)`")?;
            return Ok(if ts.is_empty() { Type::Unit } else { Type::Tuple(ts) });
        }
        // type fonction `fn(T, …) -> R` (closure idiom, 51300).
        if self.is(&TokenKind::Fn) {
            self.adv();
            self.expect(&TokenKind::LParen, "`(`")?;
            let mut ps = Vec::new();
            while !self.is(&TokenKind::RParen) {
                ps.push(self.parse_type()?);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::RParen, "`)`")?;
            let ret = if self.eat(&TokenKind::Arrow) { self.parse_type()? } else { Type::Unit };
            return Ok(Type::Fn(ps, Box::new(ret)));
        }
        // [50600 C3] entier en position d'**argument de type** (`GenericArray[u8, 32]`) → const-générique,
        // émis littéralement au lowering. Réutilise `Extern` (token brut), pas de nouvelle variante de type.
        if let TokenKind::Int(n) = self.cur() {
            let s = n.to_string();
            self.adv();
            return Ok(Type::Extern(s));
        }
        let name = self.ident()?;
        if let Some(t) = numeric_type_by_name(&name) {
            return Ok(t);
        }
        match name.as_str() {
            "i64" => Ok(Type::I64),
            "f64" => Ok(Type::F64),
            "bool" => Ok(Type::Bool),
            "str" => Ok(Type::Str),
            "char" => Ok(Type::Char),
            "unit" => Ok(Type::Unit),
            "list" => {
                self.expect(&TokenKind::LBracket, "`[`")?;
                let t = self.parse_type()?;
                self.expect(&TokenKind::RBracket, "`]`")?;
                Ok(Type::List(Box::new(t)))
            }
            "set" => {
                self.expect(&TokenKind::LBracket, "`[`")?;
                let t = self.parse_type()?;
                self.expect(&TokenKind::RBracket, "`]`")?;
                Ok(Type::Set(Box::new(t)))
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
                if self.eat(&TokenKind::Comma) {
                    let e = self.parse_type()?;
                    self.expect(&TokenKind::RBracket, "`]`")?;
                    Ok(Type::ResultE(Box::new(t), Box::new(e)))
                } else {
                    self.expect(&TokenKind::RBracket, "`]`")?;
                    Ok(Type::Result(Box::new(t)))
                }
            }
            // tuple `(A, B, …)` — non géré ici si la grammaire `.iri` n'ouvre pas par `(` ; les tuples
            // arrivent via la projection bind. Nom inconnu → type externe (résolu après coup).
            // Nom = paramètre de type du bloc courant (`T` dans `extern type Point[T]:`) → `Generic` (V12).
            other if self.cur_params.iter().any(|p| p == other) => Ok(Type::Generic(other.to_string())),
            // `Name[args]` (V12, [24000]) → type externe **générique appliqué** (`ExternApp`).
            other => {
                if self.eat(&TokenKind::LBracket) {
                    let mut args = Vec::new();
                    while !self.is(&TokenKind::RBracket) {
                        args.push(self.parse_type()?);
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RBracket, "`]`")?;
                    Ok(Type::ExternApp(other.to_string(), args))
                } else {
                    Ok(Type::Extern(other.to_string()))
                }
            }
        }
    }
}

fn resolve_fn(f: &mut ExternFn, opaque: &std::collections::HashMap<String, String>) {
    for p in &mut f.params {
        resolve_extern(p, opaque);
    }
    resolve_extern(&mut f.ret, opaque);
}

fn resolve_extern(t: &mut Type, types: &std::collections::HashMap<String, String>) {
    match t {
        Type::Extern(name) => {
            if let Some(path) = types.get(name) {
                *name = path.clone();
            }
        }
        // `ExternApp(name, args)` (V12) : résoudre le nom → chemin Rust ET récurser dans les args.
        Type::ExternApp(name, args) => {
            if let Some(path) = types.get(name) {
                *name = path.clone();
            }
            for a in args {
                resolve_extern(a, types);
            }
        }
        Type::List(e) | Type::Option(e) | Type::Result(e) | Type::Set(e) => resolve_extern(e, types),
        Type::ResultE(t, e) => {
            resolve_extern(t, types);
            resolve_extern(e, types);
        }
        Type::Dict(k, v) => {
            resolve_extern(k, types);
            resolve_extern(v, types);
        }
        _ => {}
    }
}
