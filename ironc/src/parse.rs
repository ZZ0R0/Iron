//! [10200] Parser + AST — réel (planify-2.2).
//!
//! Descente récursive + expressions en précédence (Pratt). Attribue les `NodeId`.
//! Blocs Python `:` (incrément 1). Accolades `{}` + `for`/collections/struct : incréments suivants.

use crate::ir::*;

pub fn parse(tokens: Vec<Token>) -> CResult<Ast> {
    parse_with_id(tokens, 0).map(|(ast, _)| ast)
}

/// Parse en continuant la numérotation des `NodeId` à partir de `start_id` ; renvoie l'AST + le prochain
/// id libre. Permet de fusionner plusieurs modules (70200) sans collision d'ids (clés des tables annexes).
pub fn parse_with_id(tokens: Vec<Token>, start_id: u32) -> CResult<(Ast, u32)> {
    let mut p = Parser::new(tokens);
    p.next_id = start_id;
    let ast = p.program()?;
    Ok((ast, p.next_id))
}

struct Parser {
    toks: Vec<Token>,
    i: usize,
    next_id: u32,
    /// Types opaques déclarés `extern type Name = "rust::path"` (50400) : nom → chemin Rust.
    extern_types: Vec<(String, String)>,
}

fn same(a: &TokenKind, b: &TokenKind) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

/// Opérateur d'une affectation augmentée `op=` (10700) → `BinOp` ; `None` sinon.
fn aug_op(t: &TokenKind) -> Option<BinOp> {
    Some(match t {
        TokenKind::PlusEq => BinOp::Add,
        TokenKind::MinusEq => BinOp::Sub,
        TokenKind::StarEq => BinOp::Mul,
        TokenKind::SlashEq => BinOp::Div,
        TokenKind::PercentEq => BinOp::Rem,
        _ => return None,
    })
}

impl Parser {
    fn new(toks: Vec<Token>) -> Self {
        Parser { toks, i: 0, next_id: 0, extern_types: Vec::new() }
    }

    fn fresh(&mut self) -> NodeId {
        let n = self.next_id;
        self.next_id += 1;
        NodeId(n)
    }

    fn cur(&self) -> &TokenKind {
        &self.toks[self.i].kind
    }
    fn cur_span(&self) -> Span {
        self.toks[self.i].span
    }
    fn adv(&mut self) {
        if self.i + 1 < self.toks.len() {
            self.i += 1;
        }
    }
    fn nth(&self, k: usize) -> &TokenKind {
        let i = (self.i + k).min(self.toks.len() - 1);
        &self.toks[i].kind
    }
    fn is(&self, k: &TokenKind) -> bool {
        same(self.cur(), k)
    }
    fn eat(&mut self, k: &TokenKind) -> bool {
        if self.is(k) {
            self.adv();
            true
        } else {
            false
        }
    }
    fn err(&self, msg: impl Into<String>) -> CompileError {
        CompileError { span: Some(self.cur_span()), msg: msg.into() }
    }
    fn expect(&mut self, k: &TokenKind, what: &str) -> CResult<()> {
        if self.is(k) {
            self.adv();
            Ok(())
        } else {
            Err(self.err(format!("attendu {what}, trouvé {:?}", self.cur())))
        }
    }
    fn span_from(&self, start_i: usize) -> Span {
        let a = self.toks[start_i].span;
        let end = self.toks[self.i.saturating_sub(1).max(start_i)].span.end;
        Span::new(a.file, a.start, end)
    }
    fn node<K>(&mut self, start_i: usize, id: NodeId, kind: K) -> Node<K> {
        Node { id, span: self.span_from(start_i), kind }
    }

    // ---------------------------------------------------------------- items

    fn program(&mut self) -> CResult<Ast> {
        let mut items = Vec::new();
        let mut imports = Vec::new();
        let mut modules = Vec::new();
        loop {
            while self.is(&TokenKind::Newline) {
                self.adv();
            }
            if self.is(&TokenKind::Eof) {
                break;
            }
            if matches!(self.cur(), TokenKind::Ident(s) if s == "use") {
                self.adv();
                if let TokenKind::Str(p) = self.cur() {
                    imports.push(p.clone());
                    self.adv();
                    self.stmt_end()?;
                    continue;
                }
                return Err(self.err("`use` attend un chemin \"….iri\""));
            }
            if matches!(self.cur(), TokenKind::Ident(s) if s == "import") {
                self.adv();
                // `import foo` → module Iron `foo.irn` (70200).
                let name = self.ident()?;
                modules.push(name);
                self.stmt_end()?;
                continue;
            }
            if matches!(self.cur(), TokenKind::Ident(s) if s == "class") {
                // `class Name:` (surface Python) → struct + impl inhérent.
                let (st, im) = self.class_item()?;
                items.push(st);
                if let Some(im) = im {
                    items.push(im);
                }
                continue;
            }
            if matches!(self.cur(), TokenKind::Ident(s) if s == "extern") {
                self.adv();
                // `extern type Name = "rust::path"` → type opaque (50400).
                if !matches!(self.cur(), TokenKind::Ident(s) if s == "type") {
                    return Err(self.err("`extern` attend `type Name = \"chemin::rust\"`"));
                }
                self.adv();
                let name = self.ident()?;
                self.expect(&TokenKind::Assign, "`=`")?;
                let path = match self.cur() {
                    TokenKind::Str(p) => p.clone(),
                    _ => return Err(self.err("`extern type` attend un chemin Rust entre guillemets")),
                };
                self.adv();
                self.stmt_end()?;
                self.extern_types.push((name, path));
                continue;
            }
            items.push(self.item()?);
        }
        Ok(Ast { items, imports, modules, extern_types: std::mem::take(&mut self.extern_types) })
    }

    fn item(&mut self) -> CResult<Item> {
        match self.cur() {
            TokenKind::Fn => self.fn_item(),
            TokenKind::Struct => self.struct_item(),
            TokenKind::Enum => self.enum_item(),
            TokenKind::Trait => self.trait_item(),
            TokenKind::Impl => self.impl_item(),
            other => Err(self.err(format!("item attendu, trouvé {other:?}"))),
        }
    }

    /// Liste de params `(p1, p2: T, p3 = défaut, …)` → (params, defaults). `LParen` déjà consommé.
    /// Type optionnel (20500, sentinelle si absent) ; défaut optionnel `= expr` (surface Python).
    fn params(&mut self) -> CResult<(Vec<(String, Type)>, Vec<Option<Expr>>)> {
        let mut params = Vec::new();
        let mut defaults = Vec::new();
        while !self.is(&TokenKind::RParen) {
            let is_inout = self.eat(&TokenKind::Inout);
            let pname = self.ident()?;
            let mut pty = if self.eat(&TokenKind::Colon) {
                self.parse_type()?
            } else {
                Type::Generic(INFER_SENTINEL.to_string())
            };
            if is_inout {
                pty = Type::Ref(Box::new(pty));
            }
            let default = if self.eat(&TokenKind::Assign) { Some(self.expr()?) } else { None };
            params.push((pname, pty));
            defaults.push(default);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::RParen, "`)`")?;
        Ok((params, defaults))
    }

    fn fn_item(&mut self) -> CResult<Item> {
        let start = self.i;
        let id = self.fresh();
        self.expect(&TokenKind::Fn, "`fn`")?;
        let name = self.ident()?;
        let generics = self.parse_generics()?;
        self.expect(&TokenKind::LParen, "`(`")?;
        let (params, defaults) = self.params()?;
        let ret = if self.eat(&TokenKind::Arrow) {
            self.parse_type()?
        } else {
            Type::Unit
        };
        let body = self.block()?;
        Ok(self.node(start, id, ItemKind::Fn { name, generics, params, ret, body, defaults }))
    }

    fn parse_generics(&mut self) -> CResult<Generics> {
        let mut gens = Vec::new();
        if self.eat(&TokenKind::LBracket) {
            while !self.is(&TokenKind::RBracket) {
                let name = self.ident()?;
                let mut bounds = Vec::new();
                if self.eat(&TokenKind::Colon) {
                    bounds.push(self.ident()?);
                    while self.eat(&TokenKind::Plus) {
                        bounds.push(self.ident()?);
                    }
                }
                gens.push((name, bounds));
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::RBracket, "`]`")?;
        }
        Ok(gens)
    }

    fn struct_item(&mut self) -> CResult<Item> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // struct
        let name = self.ident()?;
        let generics = self.parse_generics()?;
        self.expect(&TokenKind::Colon, "`:`")?;
        self.expect(&TokenKind::Newline, "fin de ligne")?;
        self.expect(&TokenKind::Indent, "bloc indenté")?;
        let mut fields = Vec::new();
        while !self.is(&TokenKind::Dedent) && !self.is(&TokenKind::Eof) {
            if self.eat(&TokenKind::Pass) {
                self.expect(&TokenKind::Newline, "fin de ligne")?;
                continue;
            }
            let fname = self.ident()?;
            self.expect(&TokenKind::Colon, "`:`")?;
            let fty = self.parse_type()?;
            self.expect(&TokenKind::Newline, "fin de ligne")?;
            fields.push((fname, fty));
        }
        self.expect(&TokenKind::Dedent, "fin de bloc")?;
        Ok(self.node(start, id, ItemKind::Struct { name, generics, fields }))
    }

    /// `class Name:` (surface Python, P.3) → désucré en `struct` (champs) + `impl` inhérent (méthodes `def`).
    fn class_item(&mut self) -> CResult<(Item, Option<Item>)> {
        let start = self.i;
        self.adv(); // class
        let name = self.ident()?;
        let generics = self.parse_generics()?;
        self.expect(&TokenKind::Colon, "`:`")?;
        self.expect(&TokenKind::Newline, "fin de ligne")?;
        self.expect(&TokenKind::Indent, "bloc indenté")?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        while !self.is(&TokenKind::Dedent) && !self.is(&TokenKind::Eof) {
            while self.is(&TokenKind::Newline) {
                self.adv(); // lignes vides entre champs/méthodes (style Python)
            }
            if self.is(&TokenKind::Dedent) || self.is(&TokenKind::Eof) {
                break;
            }
            if self.eat(&TokenKind::Pass) {
                self.expect(&TokenKind::Newline, "fin de ligne")?;
                continue;
            }
            if self.is(&TokenKind::Fn) {
                // `def`/`fn` méthode (self implicite)
                methods.push(self.parse_method(true)?);
            } else {
                let fname = self.ident()?;
                self.expect(&TokenKind::Colon, "`:`")?;
                let fty = self.parse_type()?;
                self.expect(&TokenKind::Newline, "fin de ligne")?;
                fields.push((fname, fty));
            }
        }
        self.expect(&TokenKind::Dedent, "fin de bloc")?;
        let sid = self.fresh();
        let struct_item =
            self.node(start, sid, ItemKind::Struct { name: name.clone(), generics, fields });
        let impl_item = if methods.is_empty() {
            None
        } else {
            let iid = self.fresh();
            Some(self.node(start, iid, ItemKind::Impl { trait_name: String::new(), type_name: name, methods }))
        };
        Ok((struct_item, impl_item))
    }

    fn enum_item(&mut self) -> CResult<Item> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // enum
        let name = self.ident()?;
        let generics = self.parse_generics()?;
        self.expect(&TokenKind::Colon, "`:`")?;
        self.expect(&TokenKind::Newline, "fin de ligne")?;
        self.expect(&TokenKind::Indent, "bloc indenté")?;
        let mut variants = Vec::new();
        loop {
            while self.is(&TokenKind::Newline) {
                self.adv();
            }
            if self.is(&TokenKind::Dedent) || self.is(&TokenKind::Eof) {
                break;
            }
            let vname = self.ident()?;
            let mut fields = Vec::new();
            if self.eat(&TokenKind::LParen) {
                while !self.is(&TokenKind::RParen) {
                    fields.push(self.parse_type()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen, "`)`")?;
            }
            self.stmt_end()?;
            variants.push((vname, fields));
        }
        self.expect(&TokenKind::Dedent, "fin de bloc")?;
        Ok(self.node(start, id, ItemKind::Enum { name, generics, variants }))
    }

    fn trait_item(&mut self) -> CResult<Item> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // trait
        let name = self.ident()?;
        self.expect(&TokenKind::Colon, "`:`")?;
        self.expect(&TokenKind::Newline, "fin de ligne")?;
        self.expect(&TokenKind::Indent, "bloc indenté")?;
        let mut methods = Vec::new();
        loop {
            while self.is(&TokenKind::Newline) {
                self.adv();
            }
            if self.is(&TokenKind::Dedent) || self.is(&TokenKind::Eof) {
                break;
            }
            methods.push(self.parse_method(false)?);
        }
        self.expect(&TokenKind::Dedent, "fin de bloc")?;
        Ok(self.node(start, id, ItemKind::Trait { name, methods }))
    }

    fn impl_item(&mut self) -> CResult<Item> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // impl
        let first = self.ident()?;
        // `impl Trait for Type:` (trait) ou `impl Type:` (inhérent, surface Python) — trait vide.
        let (trait_name, type_name) =
            if self.eat(&TokenKind::For) { (first, self.ident()?) } else { (String::new(), first) };
        self.expect(&TokenKind::Colon, "`:`")?;
        self.expect(&TokenKind::Newline, "fin de ligne")?;
        self.expect(&TokenKind::Indent, "bloc indenté")?;
        let mut methods = Vec::new();
        loop {
            while self.is(&TokenKind::Newline) {
                self.adv();
            }
            if self.is(&TokenKind::Dedent) || self.is(&TokenKind::Eof) {
                break;
            }
            methods.push(self.parse_method(true)?);
        }
        self.expect(&TokenKind::Dedent, "fin de bloc")?;
        Ok(self.node(start, id, ItemKind::Impl { trait_name, type_name, methods }))
    }

    fn parse_method(&mut self, with_body: bool) -> CResult<Method> {
        self.expect(&TokenKind::Fn, "`fn`")?;
        let name = self.ident()?;
        self.expect(&TokenKind::LParen, "`(`")?;
        let mut self_param = false;
        if matches!(self.cur(), TokenKind::Ident(s) if s == "self")
            && matches!(self.nth(1), TokenKind::Comma | TokenKind::RParen)
        {
            self.adv();
            self_param = true;
            self.eat(&TokenKind::Comma);
        }
        let (params, defaults) = self.params()?; // types optionnels + défauts (surface Python)
        let ret = if self.eat(&TokenKind::Arrow) {
            self.parse_type()?
        } else {
            Type::Unit
        };
        let body = if with_body {
            Some(self.block()?)
        } else {
            self.stmt_end()?;
            None
        };
        Ok(Method { name, self_param, params, ret, body, defaults })
    }

    fn ident(&mut self) -> CResult<String> {
        if let TokenKind::Ident(s) = self.cur() {
            let s = s.clone();
            self.adv();
            Ok(s)
        } else {
            Err(self.err(format!("identifiant attendu, trouvé {:?}", self.cur())))
        }
    }

    fn parse_type(&mut self) -> CResult<Type> {
        // type tuple `(A, B, …)` (10400)
        if self.eat(&TokenKind::LParen) {
            let mut ts = Vec::new();
            while !self.is(&TokenKind::RParen) {
                ts.push(self.parse_type()?);
                if !self.eat(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(&TokenKind::RParen, "`)`")?;
            return Ok(Type::Tuple(ts));
        }
        let name = self.ident()?;
        match name.as_str() {
            "i64" => Ok(Type::I64),
            "f64" => Ok(Type::F64),
            "bool" => Ok(Type::Bool),
            "str" => Ok(Type::Str),
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
            "shared" => {
                // partage explicite (30300, I5) : `shared[T]` → `Rc<RefCell<T>>`.
                self.expect(&TokenKind::LBracket, "`[`")?;
                let t = self.parse_type()?;
                self.expect(&TokenKind::RBracket, "`]`")?;
                Ok(Type::Shared(Box::new(t)))
            }
            "set" => {
                // ensemble (20900) : `set[T]` → `HashSet<T>`.
                self.expect(&TokenKind::LBracket, "`[`")?;
                let t = self.parse_type()?;
                self.expect(&TokenKind::RBracket, "`]`")?;
                Ok(Type::Set(Box::new(t)))
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
            _ => {
                // char + zoo numérique élargi (20800)
                if name == "char" {
                    return Ok(Type::Char);
                }
                if let Some(t) = numeric_type_by_name(&name) {
                    return Ok(t);
                }
                // type opaque déclaré `extern type Name = "path"` (50400) → Type::Extern(path).
                if let Some((_, path)) = self.extern_types.iter().find(|(n, _)| n == &name) {
                    return Ok(Type::Extern(path.clone()));
                }
                if self.is(&TokenKind::LBracket) {
                    self.adv();
                    let mut args = Vec::new();
                    while !self.is(&TokenKind::RBracket) {
                        args.push(self.parse_type()?);
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(&TokenKind::RBracket, "`]`")?;
                    Ok(Type::App(name, args))
                } else {
                    Ok(Type::Struct(name))
                }
            }
        }
    }

    /// Bloc, au choix : `:` Newline Indent … Dedent (Python) **ou** `{ … }` (accolades).
    /// (§1bis : `:` ⇒ indentation, `{` ⇒ accolades. Nesting `:` dans `{}` non supporté — V0.)
    fn block(&mut self) -> CResult<Vec<Stmt>> {
        if self.eat(&TokenKind::LBrace) {
            let mut stmts = Vec::new();
            loop {
                while self.is(&TokenKind::Newline) || self.is(&TokenKind::Semicolon) {
                    self.adv();
                }
                if self.is(&TokenKind::RBrace) || self.is(&TokenKind::Eof) {
                    break;
                }
                stmts.push(self.stmt()?);
            }
            self.expect(&TokenKind::RBrace, "`}`")?;
            Ok(stmts)
        } else {
            self.expect(&TokenKind::Colon, "`:`")?;
            self.expect(&TokenKind::Newline, "fin de ligne")?;
            self.expect(&TokenKind::Indent, "bloc indenté")?;
            let mut stmts = Vec::new();
            loop {
                while self.is(&TokenKind::Newline) || self.is(&TokenKind::Semicolon) {
                    self.adv();
                }
                if self.is(&TokenKind::Dedent) || self.is(&TokenKind::Eof) {
                    break;
                }
                stmts.push(self.stmt()?);
            }
            self.expect(&TokenKind::Dedent, "fin de bloc")?;
            Ok(stmts)
        }
    }

    /// Fin d'instruction : Newline / `;` (consommés) ou `}`/Dedent/Eof (fin de bloc implicite).
    fn stmt_end(&mut self) -> CResult<()> {
        if self.is(&TokenKind::Newline) || self.is(&TokenKind::Semicolon) {
            while self.is(&TokenKind::Newline) || self.is(&TokenKind::Semicolon) {
                self.adv();
            }
            Ok(())
        } else if self.is(&TokenKind::RBrace) || self.is(&TokenKind::Dedent) || self.is(&TokenKind::Eof) {
            Ok(())
        } else {
            Err(self.err(format!("fin d'instruction attendue, trouvé {:?}", self.cur())))
        }
    }

    // ---------------------------------------------------------------- statements

    fn stmt(&mut self) -> CResult<Stmt> {
        if let TokenKind::RustVerbatim(s) = self.cur() {
            let s = s.clone();
            let start = self.i;
            let id = self.fresh();
            self.adv();
            self.stmt_end()?;
            return Ok(self.node(start, id, StmtKind::RustBlock(s)));
        }
        // `assert` (10800) : soft-keyword (comme `use`/`import`/`extern`).
        if matches!(self.cur(), TokenKind::Ident(s) if s == "assert") {
            return self.assert_stmt();
        }
        // `try`/`raise` (surface Python) : soft-keywords.
        if matches!(self.cur(), TokenKind::Ident(s) if s == "try") {
            return self.try_stmt();
        }
        if matches!(self.cur(), TokenKind::Ident(s) if s == "raise") {
            return self.raise_stmt();
        }
        match self.cur() {
            TokenKind::Return => self.return_stmt(),
            TokenKind::If => self.if_stmt(),
            TokenKind::While => self.while_stmt(),
            TokenKind::For => self.for_stmt(),
            TokenKind::Match => self.match_stmt(),
            TokenKind::Pass => self.simple(|_, id, sp| Node { id, span: sp, kind: StmtKind::Pass }),
            TokenKind::Break => self.simple(|_, id, sp| Node { id, span: sp, kind: StmtKind::Break }),
            TokenKind::Continue => {
                self.simple(|_, id, sp| Node { id, span: sp, kind: StmtKind::Continue })
            }
            _ => self.expr_or_assign(),
        }
    }

    /// Mot-clé simple suivi d'une fin de ligne (`pass`/`break`/`continue`).
    fn simple(
        &mut self,
        make: impl Fn(&mut Self, NodeId, Span) -> Stmt,
    ) -> CResult<Stmt> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // le mot-clé
        self.stmt_end()?;
        let sp = self.span_from(start);
        Ok(make(self, id, sp))
    }

    fn try_stmt(&mut self) -> CResult<Stmt> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // try
        let body = self.block()?;
        while self.is(&TokenKind::Newline) {
            self.adv();
        }
        if !matches!(self.cur(), TokenKind::Ident(s) if s == "except") {
            return Err(self.err("`try:` attend `except [nom]:`"));
        }
        self.adv(); // except
        // `except:` / `except e:` / `except Type as e:`
        let err_name = if matches!(self.cur(), TokenKind::Ident(_)) {
            let first = self.ident()?;
            if matches!(self.cur(), TokenKind::Ident(s) if s == "as") {
                self.adv();
                Some(self.ident()?)
            } else {
                Some(first)
            }
        } else {
            None
        };
        let handler = self.block()?;
        Ok(self.node(start, id, StmtKind::TryExcept { body, err_name, handler }))
    }

    fn raise_stmt(&mut self) -> CResult<Stmt> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // raise
        let value = self.expr()?;
        self.stmt_end()?;
        Ok(self.node(start, id, StmtKind::Raise { value }))
    }

    fn assert_stmt(&mut self) -> CResult<Stmt> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // assert
        let cond = self.expr()?;
        let msg = if self.eat(&TokenKind::Comma) { Some(self.expr()?) } else { None };
        self.stmt_end()?;
        Ok(self.node(start, id, StmtKind::Assert { cond, msg }))
    }

    fn return_stmt(&mut self) -> CResult<Stmt> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // return
        let value = if self.is(&TokenKind::Newline)
            || self.is(&TokenKind::Semicolon)
            || self.is(&TokenKind::RBrace)
        {
            None
        } else {
            Some(self.expr_or_tuple()?) // `return a, b` → Tuple (10400)
        };
        self.stmt_end()?;
        Ok(self.node(start, id, StmtKind::Return { value }))
    }

    fn if_stmt(&mut self) -> CResult<Stmt> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // if
        let cond = self.expr()?;
        let then = self.block()?;
        let mut elifs = Vec::new();
        while self.is(&TokenKind::Elif) {
            self.adv();
            let c = self.expr()?;
            let b = self.block()?;
            elifs.push((c, b));
        }
        let els = if self.eat(&TokenKind::Else) {
            self.block()?
        } else {
            Vec::new()
        };
        Ok(self.node(start, id, StmtKind::If { cond, then, elifs, els }))
    }

    fn while_stmt(&mut self) -> CResult<Stmt> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // while
        let cond = self.expr()?;
        let body = self.block()?;
        Ok(self.node(start, id, StmtKind::While { cond, body }))
    }

    fn for_stmt(&mut self) -> CResult<Stmt> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // for
        // `for x in …` ou `for a, b in …` (destructuring de boucle, surface Python).
        let mut names = vec![self.ident()?];
        while self.eat(&TokenKind::Comma) {
            names.push(self.ident()?);
        }
        self.expect(&TokenKind::In, "`in`")?;
        let iter = self.expr()?;
        let body = self.block()?;
        if names.len() == 1 {
            let var = names.pop().unwrap();
            return Ok(self.node(start, id, StmtKind::For { var, iter, body }));
        }
        // désucre `for a, b in it` → `for __fvN in it: a, b = __fvN`.
        let tmp = format!("__fv{}", id.0);
        let did = self.fresh();
        let tvid = self.fresh();
        let tvar = self.node(start, tvid, ExprKind::Var(tmp.clone()));
        let destr = self.node(start, did, StmtKind::Destructure { targets: names, value: tvar });
        let mut new_body = vec![destr];
        new_body.extend(body);
        Ok(self.node(start, id, StmtKind::For { var: tmp, iter, body: new_body }))
    }

    fn match_stmt(&mut self) -> CResult<Stmt> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // match
        let scrutinee = self.expr()?;
        self.expect(&TokenKind::Colon, "`:`")?;
        self.expect(&TokenKind::Newline, "fin de ligne")?;
        self.expect(&TokenKind::Indent, "bloc indenté")?;
        let mut arms = Vec::new();
        loop {
            while self.is(&TokenKind::Newline) {
                self.adv();
            }
            if self.is(&TokenKind::Dedent) || self.is(&TokenKind::Eof) {
                break;
            }
            let pattern = self.parse_pattern()?;
            // Garde optionnelle (20600) : `<motif> if <cond>:` — l'expr s'arrête sur `:`/`{`.
            let guard = if self.eat(&TokenKind::If) { Some(self.expr()?) } else { None };
            let body = self.block()?;
            arms.push(MatchArm { pattern, guard, body });
        }
        self.expect(&TokenKind::Dedent, "fin de bloc")?;
        Ok(self.node(start, id, StmtKind::Match { scrutinee, arms }))
    }

    fn parse_pattern(&mut self) -> CResult<Pattern> {
        match self.cur().clone() {
            TokenKind::Int(n) => {
                self.adv();
                Ok(Pattern::Lit(Lit::Int(n)))
            }
            TokenKind::Bool(b) => {
                self.adv();
                Ok(Pattern::Lit(Lit::Bool(b)))
            }
            TokenKind::Str(s) => {
                self.adv();
                Ok(Pattern::Lit(Lit::Str(s)))
            }
            TokenKind::Ident(name) => {
                self.adv();
                if name == "_" {
                    return Ok(Pattern::Wild);
                }
                if self.is(&TokenKind::Dot) {
                    let mut path = vec![name];
                    while self.eat(&TokenKind::Dot) {
                        path.push(self.ident()?);
                    }
                    let sub = if self.is(&TokenKind::LParen) {
                        self.parse_pattern_args()?
                    } else {
                        Vec::new()
                    };
                    Ok(Pattern::Variant { path, sub })
                } else if self.is(&TokenKind::LParen) {
                    let sub = self.parse_pattern_args()?;
                    Ok(Pattern::Variant { path: vec![name], sub })
                } else if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                    Ok(Pattern::Variant { path: vec![name], sub: Vec::new() })
                } else {
                    Ok(Pattern::Var(name))
                }
            }
            TokenKind::LParen => {
                // motif tuple `(a, b)` (10400)
                self.adv();
                let mut subs = Vec::new();
                while !self.is(&TokenKind::RParen) {
                    subs.push(self.parse_pattern()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RParen, "`)`")?;
                Ok(Pattern::Tuple(subs))
            }
            other => Err(self.err(format!("motif attendu, trouvé {other:?}"))),
        }
    }

    fn parse_pattern_args(&mut self) -> CResult<Vec<Pattern>> {
        self.expect(&TokenKind::LParen, "`(`")?;
        let mut subs = Vec::new();
        while !self.is(&TokenKind::RParen) {
            subs.push(self.parse_pattern()?);
            if !self.eat(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(&TokenKind::RParen, "`)`")?;
        Ok(subs)
    }

    /// `lhs = rhs` (liaison/affectation) ou expression-instruction.
    fn expr_or_assign(&mut self) -> CResult<Stmt> {
        let start = self.i;
        let id = self.fresh();
        let lhs = self.expr()?;
        // Déstructuring `a, b = …` (10400) : cible = liste d'identifiants.
        if self.is(&TokenKind::Comma) {
            let mut targets = match &lhs.kind {
                ExprKind::Var(n) => vec![n.clone()],
                _ => return Err(self.err("cible de déstructuration : identifiant attendu")),
            };
            while self.eat(&TokenKind::Comma) {
                let t = self.expr()?;
                match t.kind {
                    ExprKind::Var(n) => targets.push(n),
                    _ => return Err(self.err("cible de déstructuration : identifiant attendu")),
                }
            }
            self.expect(&TokenKind::Assign, "`=`")?;
            let value = self.expr_or_tuple()?;
            self.stmt_end()?;
            return Ok(self.node(start, id, StmtKind::Destructure { targets, value }));
        }
        // affectation augmentée `x op= e` → `x = x op e` (10700).
        if let Some(op) = aug_op(self.cur()) {
            self.adv();
            let rhs = self.expr()?;
            self.stmt_end()?;
            let bid = self.fresh();
            let value =
                self.node(start, bid, ExprKind::Binary { op, lhs: Box::new(lhs.clone()), rhs: Box::new(rhs) });
            let kind = match &lhs.kind {
                ExprKind::Var(_) | ExprKind::Field { .. } | ExprKind::Index { .. } => {
                    StmtKind::Assign { target: lhs, value }
                }
                _ => return Err(self.err("cible d'affectation augmentée invalide")),
            };
            return Ok(self.node(start, id, kind));
        }
        if self.eat(&TokenKind::Assign) {
            let value = self.expr()?;
            self.stmt_end()?;
            let kind = match lhs.kind {
                ExprKind::Var(name) => StmtKind::Let { name, value },
                ExprKind::Field { .. } | ExprKind::Index { .. } => {
                    StmtKind::Assign { target: lhs, value }
                }
                _ => return Err(self.err("cible d'affectation invalide")),
            };
            Ok(self.node(start, id, kind))
        } else {
            self.stmt_end()?;
            Ok(self.node(start, id, StmtKind::ExprStmt(lhs)))
        }
    }

    // ---------------------------------------------------------------- expressions (Pratt)

    fn expr(&mut self) -> CResult<Expr> {
        let start = self.i;
        let e = self.or_expr()?;
        // ternaire `a if cond else b` (10800) : `if` infixe (le `if` d'instruction est en tête).
        if self.is(&TokenKind::If) {
            self.adv();
            let cond = self.or_expr()?;
            self.expect(&TokenKind::Else, "`else` (ternaire)")?;
            let els = self.expr()?; // associatif à droite
            let id = self.fresh();
            return Ok(self.node(
                start,
                id,
                ExprKind::Ternary { cond: Box::new(cond), then: Box::new(e), els: Box::new(els) },
            ));
        }
        Ok(e)
    }

    /// Découpe une f-string brute (10500) en fragments littéraux + sous-expressions (`{…}` apparié,
    /// `{{`/`}}` = accolades littérales).
    fn parse_fstring(&mut self, raw: &str) -> CResult<Vec<FStringPart>> {
        let mut parts = Vec::new();
        let mut lit = String::new();
        let chars: Vec<char> = raw.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            match chars[i] {
                '{' if chars.get(i + 1) == Some(&'{') => {
                    lit.push('{');
                    i += 2;
                }
                '}' if chars.get(i + 1) == Some(&'}') => {
                    lit.push('}');
                    i += 2;
                }
                '{' => {
                    if !lit.is_empty() {
                        parts.push(FStringPart::Lit(std::mem::take(&mut lit)));
                    }
                    let mut depth = 1;
                    let mut src = String::new();
                    i += 1;
                    while i < chars.len() && depth > 0 {
                        match chars[i] {
                            '{' => depth += 1,
                            '}' => {
                                depth -= 1;
                                if depth == 0 {
                                    i += 1;
                                    break;
                                }
                            }
                            _ => {}
                        }
                        if depth > 0 {
                            src.push(chars[i]);
                        }
                        i += 1;
                    }
                    if depth != 0 {
                        return Err(self.err("f-string : `{` non fermé"));
                    }
                    let e = self.parse_sub_expr(&src)?;
                    parts.push(FStringPart::Expr(Box::new(e)));
                }
                '}' => return Err(self.err("f-string : `}` non apparié")),
                c => {
                    lit.push(c);
                    i += 1;
                }
            }
        }
        if !lit.is_empty() {
            parts.push(FStringPart::Lit(lit));
        }
        Ok(parts)
    }

    /// Parse une sous-expression de f-string en threadant la numérotation des `NodeId`.
    fn parse_sub_expr(&mut self, src: &str) -> CResult<Expr> {
        let toks = crate::lex::lex(src, 0)?;
        let mut sub = Parser::new(toks);
        sub.next_id = self.next_id;
        let e = sub.expr()?;
        self.next_id = sub.next_id;
        Ok(e)
    }

    /// Queue de compréhension (10600) : `for <pat> in <iter> [if <cond>]` après le corps `body`.
    fn parse_comp(&mut self, start: usize, body: Expr, kind: CompKind) -> CResult<Expr> {
        self.expect(&TokenKind::For, "`for`")?;
        let var = self.parse_pattern()?;
        self.expect(&TokenKind::In, "`in`")?;
        let iter = self.or_expr()?; // or_expr : le `if` filtre n'est pas un ternaire
        let cond = if self.eat(&TokenKind::If) { Some(Box::new(self.or_expr()?)) } else { None };
        let id = self.fresh();
        Ok(self.node(start, id, ExprKind::Comp { kind, body: Box::new(body), var, iter: Box::new(iter), cond }))
    }

    /// Parse une expression ; si suivie de `,`, collecte un `Tuple` (retour multiple / RHS déstructuré, 10400).
    fn expr_or_tuple(&mut self) -> CResult<Expr> {
        let start = self.i;
        let first = self.expr()?;
        if !self.is(&TokenKind::Comma) {
            return Ok(first);
        }
        let id = self.fresh();
        let mut items = vec![first];
        while self.eat(&TokenKind::Comma) {
            if self.is(&TokenKind::Newline) || self.is(&TokenKind::Semicolon) || self.is(&TokenKind::RParen) {
                break;
            }
            items.push(self.expr()?);
        }
        Ok(self.node(start, id, ExprKind::Tuple(items)))
    }

    fn or_expr(&mut self) -> CResult<Expr> {
        let start = self.i;
        let mut lhs = self.and_expr()?;
        while self.is(&TokenKind::Or) {
            self.adv();
            let id = self.fresh();
            let rhs = self.and_expr()?;
            lhs = self.node(start, id, ExprKind::Binary { op: BinOp::Or, lhs: Box::new(lhs), rhs: Box::new(rhs) });
        }
        Ok(lhs)
    }

    fn and_expr(&mut self) -> CResult<Expr> {
        let start = self.i;
        let mut lhs = self.not_expr()?;
        while self.is(&TokenKind::And) {
            self.adv();
            let id = self.fresh();
            let rhs = self.not_expr()?;
            lhs = self.node(start, id, ExprKind::Binary { op: BinOp::And, lhs: Box::new(lhs), rhs: Box::new(rhs) });
        }
        Ok(lhs)
    }

    fn not_expr(&mut self) -> CResult<Expr> {
        if self.is(&TokenKind::Not) {
            let start = self.i;
            let id = self.fresh();
            self.adv();
            let e = self.not_expr()?;
            return Ok(self.node(start, id, ExprKind::Unary { op: UnOp::Not, expr: Box::new(e) }));
        }
        self.cmp_expr()
    }

    fn cmp_expr(&mut self) -> CResult<Expr> {
        let start = self.i;
        // Collecte la chaîne complète `o0 op0 o1 op1 o2 …` (10300, comparaison chaînée).
        let mut operands = vec![self.add_expr()?];
        let mut ops: Vec<BinOp> = Vec::new();
        loop {
            let op = match self.cur() {
                TokenKind::EqEq => BinOp::Eq,
                TokenKind::NotEq => BinOp::Ne,
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Le => BinOp::Le,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::Ge => BinOp::Ge,
                TokenKind::In => BinOp::In,
                // `not in` (10800) : on saute `not`, le `in` est consommé par l'adv ci-dessous.
                TokenKind::Not if matches!(self.nth(1), TokenKind::In) => {
                    self.adv();
                    BinOp::NotIn
                }
                _ => break,
            };
            self.adv();
            ops.push(op);
            operands.push(self.add_expr()?);
        }
        if ops.is_empty() {
            return Ok(operands.pop().unwrap());
        }
        // `a op b op c` ≡ `(a op b) and (b op c)` ; opérande médian dupliqué (effet de bord → évalué
        // deux fois : limitation notée, OK pour des opérandes purs `1 < x < 10`).
        let mut result: Option<Expr> = None;
        for (k, op) in ops.iter().enumerate() {
            let l = operands[k].clone();
            let r = operands[k + 1].clone();
            let id = self.fresh();
            let cmp = self.node(start, id, ExprKind::Binary { op: *op, lhs: Box::new(l), rhs: Box::new(r) });
            result = Some(match result {
                None => cmp,
                Some(prev) => {
                    let aid = self.fresh();
                    self.node(start, aid, ExprKind::Binary { op: BinOp::And, lhs: Box::new(prev), rhs: Box::new(cmp) })
                }
            });
        }
        Ok(result.unwrap())
    }

    fn add_expr(&mut self) -> CResult<Expr> {
        let start = self.i;
        let mut lhs = self.mul_expr()?;
        loop {
            let op = match self.cur() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.adv();
            let id = self.fresh();
            let rhs = self.mul_expr()?;
            lhs = self.node(start, id, ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) });
        }
        Ok(lhs)
    }

    fn mul_expr(&mut self) -> CResult<Expr> {
        let start = self.i;
        let mut lhs = self.pow_expr()?;
        loop {
            let op = match self.cur() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::SlashSlash => BinOp::FloorDiv, // division entière plancher (10700)
                TokenKind::Percent => BinOp::Rem,
                _ => break,
            };
            self.adv();
            let id = self.fresh();
            let rhs = self.pow_expr()?;
            lhs = self.node(start, id, ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) });
        }
        Ok(lhs)
    }

    /// Puissance `**` (10700) — au-dessus de `* / // %`, gauche-associatif (simplification : `2**3**2`
    /// = `(2**3)**2` ; le cas courant `x ** 2` est correct).
    fn pow_expr(&mut self) -> CResult<Expr> {
        let start = self.i;
        let mut lhs = self.unary_expr()?;
        while self.is(&TokenKind::StarStar) {
            self.adv();
            let id = self.fresh();
            let rhs = self.unary_expr()?;
            lhs = self.node(start, id, ExprKind::Binary { op: BinOp::Pow, lhs: Box::new(lhs), rhs: Box::new(rhs) });
        }
        Ok(lhs)
    }

    fn unary_expr(&mut self) -> CResult<Expr> {
        if self.is(&TokenKind::Minus) {
            let start = self.i;
            let id = self.fresh();
            self.adv();
            let e = self.unary_expr()?;
            return Ok(self.node(start, id, ExprKind::Unary { op: UnOp::Neg, expr: Box::new(e) }));
        }
        self.postfix_expr()
    }

    fn postfix_expr(&mut self) -> CResult<Expr> {
        let start = self.i;
        let mut e = self.primary()?;
        loop {
            match self.cur() {
                TokenKind::LParen => {
                    self.adv();
                    if matches!(self.cur(), TokenKind::Ident(_))
                        && matches!(self.nth(1), TokenKind::Assign)
                        && matches!(e.kind, ExprKind::Var(_))
                    {
                        // arguments nommés sur un identifiant → construction de struct / appel-kwargs `Name(champ=expr)`
                        let name = match &e.kind {
                            ExprKind::Var(n) => n.clone(),
                            _ => unreachable!(),
                        };
                        let mut fields = Vec::new();
                        while !self.is(&TokenKind::RParen) {
                            let fname = self.ident()?;
                            self.expect(&TokenKind::Assign, "`=`")?;
                            let v = self.expr()?;
                            fields.push((fname, v));
                            if !self.eat(&TokenKind::Comma) {
                                break;
                            }
                        }
                        self.expect(&TokenKind::RParen, "`)`")?;
                        let id = self.fresh();
                        e = self.node(start, id, ExprKind::StructLit { name, fields });
                    } else {
                        let mut args = Vec::new();
                        while !self.is(&TokenKind::RParen) {
                            // arguments nommés sur méthode / appel mixte : non supportés (limite V7).
                            if matches!(self.cur(), TokenKind::Ident(_)) && matches!(self.nth(1), TokenKind::Assign) {
                                return Err(self.err(
                                    "argument nommé non supporté ici (méthode ou appel mixte) — utilisez le positionnel : obj.methode(val) / sorted(xs)",
                                ));
                            }
                            args.push(self.expr()?);
                            if !self.eat(&TokenKind::Comma) {
                                break;
                            }
                        }
                        self.expect(&TokenKind::RParen, "`)`")?;
                        let id = self.fresh();
                        e = self.node(start, id, ExprKind::Call { callee: Box::new(e), args });
                    }
                }
                TokenKind::LBracket => {
                    self.adv();
                    // slice `xs[lo:hi]` (bornes optionnelles, 10800) vs index `xs[i]`.
                    let lo = if self.is(&TokenKind::Colon) { None } else { Some(self.expr()?) };
                    if self.eat(&TokenKind::Colon) {
                        let hi = if self.is(&TokenKind::RBracket) { None } else { Some(self.expr()?) };
                        self.expect(&TokenKind::RBracket, "`]`")?;
                        let id = self.fresh();
                        e = self.node(
                            start,
                            id,
                            ExprKind::Slice { base: Box::new(e), lo: lo.map(Box::new), hi: hi.map(Box::new) },
                        );
                    } else {
                        let idx = match lo {
                            Some(x) => x,
                            None => return Err(self.err("index vide")),
                        };
                        self.expect(&TokenKind::RBracket, "`]`")?;
                        let id = self.fresh();
                        e = self.node(start, id, ExprKind::Index { base: Box::new(e), idx: Box::new(idx) });
                    }
                }
                TokenKind::Dot => {
                    self.adv();
                    // `t.0` (index de tuple, 10400) ou `s.field`.
                    let name = match self.cur() {
                        TokenKind::Int(n) => {
                            let s = n.to_string();
                            self.adv();
                            s
                        }
                        _ => self.ident()?,
                    };
                    let id = self.fresh();
                    e = self.node(start, id, ExprKind::Field { base: Box::new(e), name });
                }
                TokenKind::Question => {
                    self.adv();
                    let id = self.fresh();
                    e = self.node(start, id, ExprKind::Try { inner: Box::new(e) });
                }
                _ => break,
            }
        }
        Ok(e)
    }

    fn primary(&mut self) -> CResult<Expr> {
        let start = self.i;
        let id = self.fresh();
        match self.cur().clone() {
            TokenKind::Int(n) => {
                self.adv();
                Ok(self.node(start, id, ExprKind::Lit(Lit::Int(n))))
            }
            TokenKind::Float(f) => {
                self.adv();
                Ok(self.node(start, id, ExprKind::Lit(Lit::Float(f))))
            }
            TokenKind::Bool(b) => {
                self.adv();
                Ok(self.node(start, id, ExprKind::Lit(Lit::Bool(b))))
            }
            TokenKind::Str(s) => {
                self.adv();
                Ok(self.node(start, id, ExprKind::Lit(Lit::Str(s))))
            }
            TokenKind::FStr(raw) => {
                self.adv();
                let parts = self.parse_fstring(&raw)?;
                Ok(self.node(start, id, ExprKind::FString(parts)))
            }
            TokenKind::Char(c) => {
                self.adv();
                Ok(self.node(start, id, ExprKind::Lit(Lit::Char(c))))
            }
            TokenKind::Ident(name) => {
                self.adv();
                Ok(self.node(start, id, ExprKind::Var(name)))
            }
            TokenKind::Lambda => {
                // `lambda x, y: expr` (V6, 20700) — corps mono-expression, types inférés par Rust.
                self.adv();
                let mut params = Vec::new();
                if !self.is(&TokenKind::Colon) {
                    loop {
                        params.push(self.ident()?);
                        if !self.eat(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&TokenKind::Colon, "`:` (corps de la lambda)")?;
                let body = self.expr()?;
                Ok(self.node(start, id, ExprKind::Lambda { params, body: Box::new(body) }))
            }
            TokenKind::LParen => {
                self.adv();
                if self.eat(&TokenKind::RParen) {
                    return Ok(self.node(start, id, ExprKind::Tuple(vec![])));
                }
                let first = self.expr()?;
                if self.is(&TokenKind::Comma) {
                    // tuple `(a, b, …)` ou tuple-1 `(a,)` (10400).
                    let mut items = vec![first];
                    while self.eat(&TokenKind::Comma) {
                        if self.is(&TokenKind::RParen) {
                            break;
                        }
                        items.push(self.expr()?);
                    }
                    self.expect(&TokenKind::RParen, "`)`")?;
                    return Ok(self.node(start, id, ExprKind::Tuple(items)));
                }
                self.expect(&TokenKind::RParen, "`)`")?;
                Ok(first)
            }
            TokenKind::LBracket => {
                self.adv();
                if self.is(&TokenKind::RBracket) {
                    self.adv();
                    return Ok(self.node(start, id, ExprKind::ListLit { items: Vec::new() }));
                }
                let first = self.expr()?;
                if self.is(&TokenKind::For) {
                    // compréhension de liste `[body for x in iter if cond]` (10600)
                    let comp = self.parse_comp(start, first, CompKind::List)?;
                    self.expect(&TokenKind::RBracket, "`]`")?;
                    return Ok(comp);
                }
                let mut items = vec![first];
                while self.eat(&TokenKind::Comma) {
                    if self.is(&TokenKind::RBracket) {
                        break;
                    }
                    items.push(self.expr()?);
                }
                self.expect(&TokenKind::RBracket, "`]`")?;
                Ok(self.node(start, id, ExprKind::ListLit { items }))
            }
            TokenKind::LBrace => {
                self.adv();
                if self.is(&TokenKind::RBrace) {
                    self.adv();
                    return Ok(self.node(start, id, ExprKind::DictLit { pairs: Vec::new() })); // `{}` vide = dict
                }
                let first = self.expr()?;
                if self.eat(&TokenKind::Colon) {
                    let v = self.expr()?;
                    if self.is(&TokenKind::For) {
                        // compréhension de dict `{k: v for x in xs}` (10600) → body = Tuple([k, v]).
                        let tid = self.fresh();
                        let tup = self.node(start, tid, ExprKind::Tuple(vec![first, v]));
                        let comp = self.parse_comp(start, tup, CompKind::Dict)?;
                        self.expect(&TokenKind::RBrace, "`}`")?;
                        return Ok(comp);
                    }
                    // dict `{k: v, …}`
                    let mut pairs = vec![(first, v)];
                    while self.eat(&TokenKind::Comma) {
                        if self.is(&TokenKind::RBrace) {
                            break;
                        }
                        let k = self.expr()?;
                        self.expect(&TokenKind::Colon, "`:`")?;
                        let v = self.expr()?;
                        pairs.push((k, v));
                    }
                    self.expect(&TokenKind::RBrace, "`}`")?;
                    Ok(self.node(start, id, ExprKind::DictLit { pairs }))
                } else if self.is(&TokenKind::For) {
                    // compréhension de set `{e for x in xs}` (10600)
                    let comp = self.parse_comp(start, first, CompKind::Set)?;
                    self.expect(&TokenKind::RBrace, "`}`")?;
                    Ok(comp)
                } else {
                    // set `{a, b, …}` → désucré en `set([a, b, …])` (20900).
                    let mut items = vec![first];
                    while self.eat(&TokenKind::Comma) {
                        if self.is(&TokenKind::RBrace) {
                            break;
                        }
                        items.push(self.expr()?);
                    }
                    self.expect(&TokenKind::RBrace, "`}`")?;
                    let lid = self.fresh();
                    let list = self.node(start, lid, ExprKind::ListLit { items });
                    let cid = self.fresh();
                    let callee = self.node(start, cid, ExprKind::Var("set".to_string()));
                    Ok(self.node(start, id, ExprKind::Call { callee: Box::new(callee), args: vec![list] }))
                }
            }
            other => Err(self.err(format!("expression attendue, trouvé {other:?}"))),
        }
    }
}
