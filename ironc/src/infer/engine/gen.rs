//! [21800] Génération de contraintes (V10) : parcourt chaque corps de fonction et émet les `Constraint`
//! qui fixent les types. Mode *synth* : `expr` calcule un type (var ou concret) et émet les contraintes en
//! chemin. Les **appels** (fns/méthodes/externes) propagent les types vers les params du callee (inférence
//! inter-fonction). Cas non couverts → var fraîche (sûr : reste libre → générique/défaut, jamais faux).

#![allow(dead_code)]

use super::assume::{Env, FnSig};
use super::constraint::{Constraint, TraitBound};
use super::subst::Subst;
use crate::ir::*;
use std::collections::HashMap;

type Scope = HashMap<String, Type>;

pub struct Gen<'a> {
    env: &'a Env,
    subst: &'a mut Subst,
    pub cs: Vec<Constraint>,
}

impl<'a> Gen<'a> {
    pub fn new(env: &'a Env, subst: &'a mut Subst) -> Self {
        Gen { env, subst, cs: Vec::new() }
    }

    fn fresh(&mut self) -> Type {
        self.subst.fresh()
    }
    fn eq(&mut self, a: Type, b: Type, sp: Span) {
        self.cs.push(Constraint::Eq(a, b, sp));
    }
    fn req(&mut self, t: Type, b: TraitBound, sp: Span) {
        self.cs.push(Constraint::Trait(t, b, sp));
    }

    /// Génère les contraintes de toutes les fonctions + méthodes (programme entier).
    pub fn program(&mut self, ast: &Ast) {
        for it in &ast.items {
            match &it.kind {
                ItemKind::Fn { name, params, body, .. } => {
                    let sig = self.env.fns[name].clone();
                    let mut scope: Scope = HashMap::new();
                    for ((pn, _), pt) in params.iter().zip(&sig.params) {
                        scope.insert(pn.clone(), deref(pt)); // inout `Ref(T)` → `T` (le corps voit la valeur)
                    }
                    self.block(&mut scope, &sig.ret, body);
                }
                ItemKind::Impl { type_name, methods, .. } => {
                    for m in methods {
                        let Some(sig) = self.env.methods.get(&(type_name.clone(), m.name.clone())).cloned() else {
                            continue;
                        };
                        let mut scope: Scope = HashMap::new();
                        let mut pi = 0;
                        if m.self_param {
                            scope.insert("self".into(), sig.params[0].clone());
                            pi = 1;
                        }
                        for (mp, pt) in m.params.iter().zip(sig.params[pi..].iter()) {
                            scope.insert(mp.0.clone(), deref(pt));
                        }
                        if let Some(b) = &m.body {
                            self.block(&mut scope, &sig.ret, b);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn block(&mut self, scope: &mut Scope, ret: &Type, body: &[Stmt]) {
        for s in body {
            self.stmt(scope, ret, s);
        }
    }

    fn stmt(&mut self, scope: &mut Scope, ret: &Type, s: &Stmt) {
        match &s.kind {
            StmtKind::Let { name, value } => {
                let t = self.expr(scope, value);
                // réassignation d'un local déjà connu → unifier (accumulateur) ; sinon nouveau local.
                if let Some(prev) = scope.get(name).cloned() {
                    self.eq(prev, t, s.span);
                } else {
                    scope.insert(name.clone(), t);
                }
            }
            StmtKind::Assign { target, value } => {
                let tv = self.expr(scope, value);
                let tt = self.expr(scope, target);
                self.eq(tt, tv, s.span);
            }
            StmtKind::Destructure { value, .. } => {
                let _ = self.expr(scope, value);
            }
            StmtKind::Return { value: Some(v) } => {
                let tv = self.expr(scope, v);
                self.eq(tv, ret.clone(), s.span);
            }
            StmtKind::Return { value: None } => {}
            StmtKind::ExprStmt(e) | StmtKind::Yield(e) => {
                self.expr(scope, e);
            }
            StmtKind::If { cond, then, elifs, els } => {
                let tc = self.expr(scope, cond);
                self.eq(tc, Type::Bool, cond.span);
                self.block(scope, ret, then);
                for (c, b) in elifs {
                    let t = self.expr(scope, c);
                    self.eq(t, Type::Bool, c.span);
                    self.block(scope, ret, b);
                }
                self.block(scope, ret, els);
            }
            StmtKind::While { cond, body } => {
                let tc = self.expr(scope, cond);
                self.eq(tc, Type::Bool, cond.span);
                self.block(scope, ret, body);
            }
            StmtKind::For { var, iter, body } => {
                let elem = self.iter_elem(scope, iter);
                scope.entry(var.clone()).or_insert(elem);
                self.block(scope, ret, body);
            }
            StmtKind::Match { scrutinee, arms } => {
                self.expr(scope, scrutinee);
                for a in arms {
                    self.block(scope, ret, &a.body);
                }
            }
            StmtKind::Assert { cond, msg } => {
                let tc = self.expr(scope, cond);
                self.eq(tc, Type::Bool, cond.span);
                if let Some(m) = msg {
                    self.expr(scope, m);
                }
            }
            StmtKind::TryExcept { body, handler, .. } => {
                self.block(scope, ret, body);
                self.block(scope, ret, handler);
            }
            StmtKind::Raise { value } => {
                self.expr(scope, value);
            }
            StmtKind::Break | StmtKind::Continue | StmtKind::Pass | StmtKind::RustBlock(_) => {}
        }
    }

    /// Type d'élément produit en itérant `iter` (`for x in iter`).
    fn iter_elem(&mut self, scope: &mut Scope, iter: &Expr) -> Type {
        // `range(...)` → i64 ; on type quand même ses args (bornes numériques) → contraint `lo`/`hi`.
        if let ExprKind::Call { callee, args } = &iter.kind {
            if matches!(&callee.kind, ExprKind::Var(n) if n == "range") {
                for a in args {
                    let t = self.expr(scope, a);
                    self.eq(t, Type::I64, a.span);
                }
                return Type::I64;
            }
        }
        let t = self.expr(scope, iter);
        match self.subst.apply(&t) {
            Type::List(e) | Type::Set(e) => *e,
            Type::Dict(k, _) => *k,
            _ => self.fresh(),
        }
    }

    fn expr(&mut self, scope: &mut Scope, e: &Expr) -> Type {
        match &e.kind {
            ExprKind::Lit(l) => match l {
                Lit::Int(_) => Type::I64,
                Lit::Float(_) => Type::F64,
                Lit::Str(_) => Type::Str,
                Lit::Bool(_) => Type::Bool,
                Lit::Char(_) => Type::Char,
            },
            ExprKind::FString(parts) => {
                for p in parts {
                    if let FStringPart::Expr(inner) = p {
                        let t = self.expr(scope, inner);
                        self.req(t, TraitBound::Display, inner.span); // interpolé ⇒ affichable (→ défaut `str`)
                    }
                }
                Type::Str
            }
            ExprKind::Var(n) => scope.get(n).cloned().unwrap_or_else(|| self.fresh()),
            ExprKind::Unary { op, expr } => {
                let t = self.expr(scope, expr);
                match op {
                    UnOp::Not => {
                        self.eq(t, Type::Bool, e.span);
                        Type::Bool
                    }
                    UnOp::Neg => t,
                }
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let tl = self.expr(scope, lhs);
                let tr = self.expr(scope, rhs);
                self.binary(op, tl, tr, e.span)
            }
            ExprKind::Call { callee, args } => self.call(scope, callee, args, e.span),
            ExprKind::StructLit { name, fields } => {
                if let Some(decl) = self.env.structs.get(name).cloned() {
                    for (fname, fexpr) in fields {
                        let ft = self.expr(scope, fexpr);
                        if let Some((_, dt)) = decl.iter().find(|(n, _)| n == fname) {
                            // struct générique (`Pair[A, B]`) : champ de type `Generic` non instancié →
                            // ne PAS unifier (sinon `Pair(left=10)` ⇒ `i64 vs A`). local.rs tranchera.
                            if !contains_generic(dt) {
                                self.eq(ft, dt.clone(), fexpr.span);
                            }
                        }
                    }
                }
                Type::Struct(name.clone())
            }
            ExprKind::Field { base, name } => {
                let tb = self.expr(scope, base);
                match self.subst.apply(&tb) {
                    Type::Struct(s) => self
                        .env
                        .structs
                        .get(&s)
                        .and_then(|fs| fs.iter().find(|(n, _)| n == name))
                        .map(|(_, t)| t.clone())
                        .unwrap_or_else(|| self.fresh()),
                    _ => self.fresh(),
                }
            }
            ExprKind::Index { base, idx } => {
                let tb = self.expr(scope, base);
                let ti = self.expr(scope, idx);
                match self.subst.apply(&tb) {
                    Type::List(el) => {
                        self.eq(ti, Type::I64, idx.span);
                        *el
                    }
                    Type::Str => {
                        self.eq(ti, Type::I64, idx.span);
                        Type::Str
                    }
                    Type::Dict(k, v) => {
                        self.eq(ti, *k, idx.span);
                        *v
                    }
                    _ => self.fresh(),
                }
            }
            ExprKind::ListLit { items } => {
                let elem = self.fresh();
                for it in items {
                    let t = self.expr(scope, it);
                    self.eq(t, elem.clone(), it.span);
                }
                Type::List(Box::new(elem))
            }
            ExprKind::DictLit { pairs } => {
                let (k, v) = (self.fresh(), self.fresh());
                for (kk, vv) in pairs {
                    let tk = self.expr(scope, kk);
                    let tv = self.expr(scope, vv);
                    self.eq(tk, k.clone(), kk.span);
                    self.eq(tv, v.clone(), vv.span);
                }
                Type::Dict(Box::new(k), Box::new(v))
            }
            ExprKind::Tuple(xs) => Type::Tuple(xs.iter().map(|x| self.expr(scope, x)).collect()),
            ExprKind::Try { inner } => {
                let t = self.expr(scope, inner);
                match self.subst.apply(&t) {
                    Type::Option(x) | Type::Result(x) => *x,
                    Type::ResultE(x, _) => *x,
                    _ => self.fresh(),
                }
            }
            ExprKind::Await(inner) => self.expr(scope, inner),
            ExprKind::Lambda { params, body } => {
                let pvars: Vec<Type> = params.iter().map(|_| self.fresh()).collect();
                for (p, v) in params.iter().zip(&pvars) {
                    scope.insert(p.clone(), v.clone());
                }
                let rt = self.expr(scope, body);
                Type::Fn(pvars, Box::new(rt))
            }
            ExprKind::Ternary { cond, then, els } => {
                let tc = self.expr(scope, cond);
                self.eq(tc, Type::Bool, cond.span);
                let tt = self.expr(scope, then);
                let te = self.expr(scope, els);
                self.eq(tt.clone(), te, e.span);
                tt
            }
            ExprKind::Slice { base, lo, hi } => {
                let tb = self.expr(scope, base);
                if let Some(l) = lo {
                    let t = self.expr(scope, l);
                    self.eq(t, Type::I64, l.span);
                }
                if let Some(h) = hi {
                    let t = self.expr(scope, h);
                    self.eq(t, Type::I64, h.span);
                }
                tb // slice d'une liste → liste, d'une str → str
            }
            ExprKind::Comp { kind, body, var, iter, cond } => {
                let elem = self.iter_elem(scope, iter);
                if let Pattern::Var(v) = var {
                    scope.entry(v.clone()).or_insert(elem);
                }
                if let Some(c) = cond {
                    self.expr(scope, c);
                }
                let bt = self.expr(scope, body);
                match kind {
                    CompKind::List => Type::List(Box::new(bt)),
                    CompKind::Set => Type::Set(Box::new(bt)),
                    CompKind::Dict => Type::Dict(Box::new(bt.clone()), Box::new(bt)),
                }
            }
        }
    }

    /// `a ⊕ b` : unifie les opérandes, requiert la borne, calcule le résultat.
    fn binary(&mut self, op: &BinOp, tl: Type, tr: Type, sp: Span) -> Type {
        use BinOp::*;
        match op {
            Add | Sub | Mul | Div | Rem | Pow | FloorDiv | BitAnd | BitOr | BitXor | Shl | Shr => {
                self.eq(tl.clone(), tr, sp);
                let b = match op {
                    Add => TraitBound::Add,
                    Sub => TraitBound::Sub,
                    Mul => TraitBound::Mul,
                    Div => TraitBound::Div,
                    _ => TraitBound::Sub, // Rem/Pow///, bit-ops : numérique (str exclu)
                };
                self.req(tl.clone(), b, sp);
                tl
            }
            Eq | Ne => {
                self.eq(tl.clone(), tr, sp);
                self.req(tl, TraitBound::Eq, sp);
                Type::Bool
            }
            Lt | Le | Gt | Ge => {
                self.eq(tl.clone(), tr, sp);
                self.req(tl, TraitBound::Ord, sp);
                Type::Bool
            }
            And | Or => {
                self.eq(tl, Type::Bool, sp);
                self.eq(tr, Type::Bool, sp);
                Type::Bool
            }
            In | NotIn => Type::Bool,
        }
    }

    fn call(&mut self, scope: &mut Scope, callee: &Expr, args: &[Expr], sp: Span) -> Type {
        // Méthode : `base.m(args)`
        if let ExprKind::Field { base, name } = &callee.kind {
            let tb = self.expr(scope, base);
            let arg_ts: Vec<Type> = args.iter().map(|a| self.expr(scope, a)).collect();
            if let Type::Struct(s) | Type::Extern(s) = self.subst.apply(&tb) {
                if let Some(sig) = self.env.methods.get(&(s, name.clone())).cloned() {
                    // params[0] = self ; aligner args sur params[1..]
                    let off = if sig.has_self { 1 } else { 0 };
                    for (at, pt) in arg_ts.iter().zip(sig.params.iter().skip(off)) {
                        self.eq(at.clone(), pt.clone(), sp);
                    }
                    return sig.ret;
                }
            }
            return self.fresh(); // méthode builtin/inconnue → local.rs tranchera
        }
        // Appel nommé
        let arg_ts: Vec<Type> = args.iter().map(|a| self.expr(scope, a)).collect();
        if let ExprKind::Var(fname) = &callee.kind {
            if let Some(t) = self.builtin_call(fname, &arg_ts, sp) {
                return t;
            }
            if let Some(sig) = self.env.fns.get(fname).cloned() {
                let (params, ret) = self.instantiate(&sig);
                for (at, pt) in arg_ts.iter().zip(&params) {
                    self.eq(at.clone(), pt.clone(), sp);
                }
                return ret;
            }
            if let Some((params, ret)) = self.env.extern_fns.get(fname).cloned() {
                for (at, pt) in arg_ts.iter().zip(&params) {
                    self.eq(at.clone(), pt.clone(), sp);
                }
                return ret;
            }
            // ctor de struct Iron par nom ? (StructLit gère `Name(field=…)` ; ici positionnel rare)
        }
        self.fresh()
    }

    /// Instancie une signature **fraîchement** à chaque appel (let-polymorphism, D6) : génériques déclarés
    /// ET métavariables de la signature → copies fraîches, de façon cohérente (un même `T`/`Var` partagé
    /// entre params et retour map vers la même fraîche). Évite le partage monomorphe entre sites d'appel
    /// (`id(7)` et `id("hi")` n'entrent pas en conflit).
    fn instantiate(&mut self, sig: &FnSig) -> (Vec<Type>, Type) {
        let mut gmap: HashMap<String, Type> = HashMap::new();
        let mut vmap: HashMap<u32, Type> = HashMap::new();
        let params: Vec<Type> =
            sig.params.iter().map(|t| self.freshen(t, &mut gmap, &mut vmap)).collect();
        let ret = self.freshen(&sig.ret, &mut gmap, &mut vmap);
        (params, ret)
    }

    /// Copie un type en remplaçant `Generic(g)`/`Var(v)` par des fraîches (maps partagés pour la cohérence).
    fn freshen(&mut self, t: &Type, gmap: &mut HashMap<String, Type>, vmap: &mut HashMap<u32, Type>) -> Type {
        match t {
            Type::Generic(g) => gmap.entry(g.clone()).or_insert_with(|| self.subst.fresh()).clone(),
            Type::Var(v) => vmap.entry(*v).or_insert_with(|| self.subst.fresh()).clone(),
            Type::Option(x) => Type::Option(Box::new(self.freshen(x, gmap, vmap))),
            Type::Result(x) => Type::Result(Box::new(self.freshen(x, gmap, vmap))),
            Type::ResultE(x, e) => {
                Type::ResultE(Box::new(self.freshen(x, gmap, vmap)), Box::new(self.freshen(e, gmap, vmap)))
            }
            Type::List(x) => Type::List(Box::new(self.freshen(x, gmap, vmap))),
            Type::Set(x) => Type::Set(Box::new(self.freshen(x, gmap, vmap))),
            Type::Shared(x) => Type::Shared(Box::new(self.freshen(x, gmap, vmap))),
            Type::SharedSync(x) => Type::SharedSync(Box::new(self.freshen(x, gmap, vmap))),
            Type::Weak(x) => Type::Weak(Box::new(self.freshen(x, gmap, vmap))),
            Type::Ref(x) => Type::Ref(Box::new(self.freshen(x, gmap, vmap))),
            Type::Dict(k, v) => {
                Type::Dict(Box::new(self.freshen(k, gmap, vmap)), Box::new(self.freshen(v, gmap, vmap)))
            }
            Type::Tuple(xs) => Type::Tuple(xs.iter().map(|x| self.freshen(x, gmap, vmap)).collect()),
            Type::App(n, xs) => {
                Type::App(n.clone(), xs.iter().map(|x| self.freshen(x, gmap, vmap)).collect())
            }
            Type::Fn(ps, r) => Type::Fn(
                ps.iter().map(|x| self.freshen(x, gmap, vmap)).collect(),
                Box::new(self.freshen(r, gmap, vmap)),
            ),
            _ => t.clone(),
        }
    }

    /// Builtins du langage (primitives) : type de résultat connu ; `None` = pas un builtin.
    fn builtin_call(&mut self, name: &str, args: &[Type], sp: Span) -> Option<Type> {
        Some(match name {
            "print" => Type::Unit,
            "len" => Type::I64,
            "range" => Type::List(Box::new(Type::I64)),
            "str" => Type::Str,
            "ok" => Type::Result(Box::new(args.first().cloned().unwrap_or_else(|| self.fresh()))),
            "some" => Type::Option(Box::new(args.first().cloned().unwrap_or_else(|| self.fresh()))),
            "err" => Type::Result(Box::new(self.fresh())),
            "abs" => args.first().cloned().unwrap_or(Type::I64),
            "min" | "max" | "sum" => {
                let t = args.first().cloned().unwrap_or_else(|| self.fresh());
                // élément d'une liste, sinon le type tel quel
                match self.subst.apply(&t) {
                    Type::List(e) => *e,
                    other => other,
                }
            }
            "sqrt" => {
                if let Some(a) = args.first() {
                    self.eq(a.clone(), Type::F64, sp);
                }
                Type::F64
            }
            _ => return None,
        })
    }
}

/// `inout x` : le corps voit la valeur `T`, pas `Ref(T)`.
fn deref(t: &Type) -> Type {
    match t {
        Type::Ref(inner) => (**inner).clone(),
        _ => t.clone(),
    }
}

/// Le type contient-il un paramètre générique (`Generic`) non instancié ?
fn contains_generic(t: &Type) -> bool {
    match t {
        Type::Generic(_) => true,
        Type::Option(x) | Type::Result(x) | Type::List(x) | Type::Set(x) | Type::Shared(x)
        | Type::SharedSync(x) | Type::Weak(x) | Type::Ref(x) => contains_generic(x),
        Type::ResultE(x, e) => contains_generic(x) || contains_generic(e),
        Type::Dict(k, v) => contains_generic(k) || contains_generic(v),
        Type::Tuple(xs) | Type::App(_, xs) => xs.iter().any(contains_generic),
        Type::Fn(ps, r) => ps.iter().any(contains_generic) || contains_generic(r),
        _ => false,
    }
}

/// Remplace les `Generic(g)` par leur instance (let-polymorphism).
fn subst_generics(t: &Type, m: &HashMap<String, Type>) -> Type {
    match t {
        Type::Generic(g) => m.get(g).cloned().unwrap_or_else(|| t.clone()),
        Type::Option(x) => Type::Option(Box::new(subst_generics(x, m))),
        Type::Result(x) => Type::Result(Box::new(subst_generics(x, m))),
        Type::ResultE(x, e) => Type::ResultE(Box::new(subst_generics(x, m)), Box::new(subst_generics(e, m))),
        Type::List(x) => Type::List(Box::new(subst_generics(x, m))),
        Type::Set(x) => Type::Set(Box::new(subst_generics(x, m))),
        Type::Shared(x) => Type::Shared(Box::new(subst_generics(x, m))),
        Type::SharedSync(x) => Type::SharedSync(Box::new(subst_generics(x, m))),
        Type::Weak(x) => Type::Weak(Box::new(subst_generics(x, m))),
        Type::Ref(x) => Type::Ref(Box::new(subst_generics(x, m))),
        Type::Dict(k, v) => Type::Dict(Box::new(subst_generics(k, m)), Box::new(subst_generics(v, m))),
        Type::Tuple(xs) => Type::Tuple(xs.iter().map(|x| subst_generics(x, m)).collect()),
        Type::App(n, xs) => Type::App(n.clone(), xs.iter().map(|x| subst_generics(x, m)).collect()),
        Type::Fn(ps, r) => {
            Type::Fn(ps.iter().map(|x| subst_generics(x, m)).collect(), Box::new(subst_generics(r, m)))
        }
        _ => t.clone(),
    }
}
