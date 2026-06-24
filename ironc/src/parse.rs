//! [10200] Parser + AST — réel (planify-2.2).
//!
//! Descente récursive + expressions en précédence (Pratt). Attribue les `NodeId`.
//! Blocs Python `:` (incrément 1). Accolades `{}` + `for`/collections/struct : incréments suivants.

use crate::ir::*;

pub fn parse(tokens: Vec<Token>) -> CResult<Ast> {
    Parser::new(tokens).program()
}

struct Parser {
    toks: Vec<Token>,
    i: usize,
    next_id: u32,
}

fn same(a: &TokenKind, b: &TokenKind) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

impl Parser {
    fn new(toks: Vec<Token>) -> Self {
        Parser { toks, i: 0, next_id: 0 }
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
            items.push(self.item()?);
        }
        Ok(Ast { items, imports })
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

    fn fn_item(&mut self) -> CResult<Item> {
        let start = self.i;
        let id = self.fresh();
        self.expect(&TokenKind::Fn, "`fn`")?;
        let name = self.ident()?;
        let generics = self.parse_generics()?;
        self.expect(&TokenKind::LParen, "`(`")?;
        let mut params = Vec::new();
        while !self.is(&TokenKind::RParen) {
            let is_inout = self.eat(&TokenKind::Inout);
            let pname = self.ident()?;
            // Type de param **optionnel** (20500) : absent → sentinelle résolue en pré-passe (i64 si
            // usage numérique, sinon générique implicite). L'annotation explicite prime.
            let mut pty = if self.eat(&TokenKind::Colon) {
                self.parse_type()?
            } else {
                Type::Generic(INFER_SENTINEL.to_string())
            };
            if is_inout {
                pty = Type::Ref(Box::new(pty));
            }
            params.push((pname, pty));
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
        let body = self.block()?;
        Ok(self.node(start, id, ItemKind::Fn { name, generics, params, ret, body }))
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
        let trait_name = self.ident()?;
        self.expect(&TokenKind::For, "`for`")?;
        let type_name = self.ident()?;
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
        let mut params = Vec::new();
        while !self.is(&TokenKind::RParen) {
            let is_inout = self.eat(&TokenKind::Inout);
            let pn = self.ident()?;
            self.expect(&TokenKind::Colon, "`:`")?;
            let mut pt = self.parse_type()?;
            if is_inout {
                pt = Type::Ref(Box::new(pt));
            }
            params.push((pn, pt));
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
        let body = if with_body {
            Some(self.block()?)
        } else {
            self.stmt_end()?;
            None
        };
        Ok(Method { name, self_param, params, ret, body })
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
            Some(self.expr()?)
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
        let var = self.ident()?;
        self.expect(&TokenKind::In, "`in`")?;
        let iter = self.expr()?;
        let body = self.block()?;
        Ok(self.node(start, id, StmtKind::For { var, iter, body }))
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
        self.or_expr()
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
        let mut lhs = self.unary_expr()?;
        loop {
            let op = match self.cur() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Rem,
                _ => break,
            };
            self.adv();
            let id = self.fresh();
            let rhs = self.unary_expr()?;
            lhs = self.node(start, id, ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) });
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
                    if matches!(self.cur(), TokenKind::Ident(_)) && matches!(self.nth(1), TokenKind::Assign) {
                        // arguments nommés → construction de struct `S(champ=expr)`
                        let name = match &e.kind {
                            ExprKind::Var(n) => n.clone(),
                            _ => return Err(self.err("construction de struct sur non-identifiant")),
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
                    let idx = self.expr()?;
                    self.expect(&TokenKind::RBracket, "`]`")?;
                    let id = self.fresh();
                    e = self.node(start, id, ExprKind::Index { base: Box::new(e), idx: Box::new(idx) });
                }
                TokenKind::Dot => {
                    self.adv();
                    let name = self.ident()?;
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
                let e = self.expr()?;
                self.expect(&TokenKind::RParen, "`)`")?;
                Ok(e)
            }
            TokenKind::LBracket => {
                self.adv();
                let mut items = Vec::new();
                while !self.is(&TokenKind::RBracket) {
                    items.push(self.expr()?);
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RBracket, "`]`")?;
                Ok(self.node(start, id, ExprKind::ListLit { items }))
            }
            TokenKind::LBrace => {
                self.adv();
                let mut pairs = Vec::new();
                while !self.is(&TokenKind::RBrace) {
                    let k = self.expr()?;
                    self.expect(&TokenKind::Colon, "`:`")?;
                    let v = self.expr()?;
                    pairs.push((k, v));
                    if !self.eat(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(&TokenKind::RBrace, "`}`")?;
                Ok(self.node(start, id, ExprKind::DictLit { pairs }))
            }
            other => Err(self.err(format!("expression attendue, trouvé {other:?}"))),
        }
    }
}
