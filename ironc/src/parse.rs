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
        loop {
            while self.is(&TokenKind::Newline) {
                self.adv();
            }
            if self.is(&TokenKind::Eof) {
                break;
            }
            items.push(self.item()?);
        }
        Ok(Ast { items })
    }

    fn item(&mut self) -> CResult<Item> {
        match self.cur() {
            TokenKind::Fn => self.fn_item(),
            TokenKind::Struct => self.struct_item(),
            other => Err(self.err(format!("item attendu (`fn`/`struct`), trouvé {other:?}"))),
        }
    }

    fn fn_item(&mut self) -> CResult<Item> {
        let start = self.i;
        let id = self.fresh();
        self.expect(&TokenKind::Fn, "`fn`")?;
        let name = self.ident()?;
        self.expect(&TokenKind::LParen, "`(`")?;
        let mut params = Vec::new();
        while !self.is(&TokenKind::RParen) {
            let pname = self.ident()?;
            self.expect(&TokenKind::Colon, "`:` (type de paramètre requis en V0)")?;
            let pty = self.parse_type()?;
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
        Ok(self.node(start, id, ItemKind::Fn { name, params, ret, body }))
    }

    fn struct_item(&mut self) -> CResult<Item> {
        let start = self.i;
        let id = self.fresh();
        self.adv(); // struct
        let name = self.ident()?;
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
        Ok(self.node(start, id, ItemKind::Struct { name, fields }))
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
            _ => Ok(Type::Struct(name)),
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
        match self.cur() {
            TokenKind::Return => self.return_stmt(),
            TokenKind::If => self.if_stmt(),
            TokenKind::While => self.while_stmt(),
            TokenKind::For => self.for_stmt(),
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
        let mut lhs = self.add_expr()?;
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
            let id = self.fresh();
            let rhs = self.add_expr()?;
            lhs = self.node(start, id, ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) });
        }
        Ok(lhs)
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
