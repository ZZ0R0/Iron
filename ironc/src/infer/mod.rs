//! [20100 + 20200] Inférence — réelle (planify-2.3).
//!
//! `local.rs` (monotypes, structs, collections) + `flow.rs` (Option/Result par flux).
//! Best-effort : annote tout `Expr` joignable ; ce que `lower` exige (type des bases d'index,
//! `Option`/`Result`) est couvert. Vérification d'annotations : incrément ultérieur.

pub(crate) mod engine; // [V10] moteur d'inférence constraint-based — résout params/retours non annotés
mod local;

use crate::ir::*;
use std::collections::{HashMap, HashSet};

/// Signatures de fonctions : nom → (types des params, type de retour **déclaré**).
pub(crate) type Sigs = HashMap<String, (Vec<Type>, Type)>;
/// Champs de structs : nom → (champ → type).
pub(crate) type Structs = HashMap<String, Vec<(String, Type)>>;
/// Enums : nom → variantes (nom de variante → types des champs tuple).
pub(crate) type Enums = HashMap<String, Vec<(String, Vec<Type>)>>;
/// Méthodes d'`impl` : (type, méthode) → type de retour (V5, inférence d'appel de méthode).
pub(crate) type Methods = HashMap<(String, String), Type>;

pub fn infer(ast: &Ast, bindings: &Bindings) -> TypeMap {
    let mut sigs = Sigs::new();
    let mut structs = Structs::new();
    let mut enums = Enums::new();
    let mut methods_tbl = Methods::new();
    // (id du nœud fn, type de retour **effectif**) — I11 par flux : `sigs`/`tm` portent l'effectif,
    // `lower` compare déclaré (AST) vs effectif (`tm`) pour savoir s'il doit emballer (`Some`/`Ok`).
    let mut fn_effs: Vec<(NodeId, Type)> = Vec::new();
    for it in &ast.items {
        match &it.kind {
            ItemKind::Fn { name, params, ret, body, .. } => {
                let eff = engine::flux::effective_ret(ret, body);
                sigs.insert(name.clone(), (params.iter().map(|(_, t)| t.clone()).collect(), eff.clone()));
                fn_effs.push((it.id, eff));
            }
            ItemKind::Struct { name, fields, .. } => {
                structs.insert(name.clone(), fields.clone());
            }
            ItemKind::Enum { name, variants, .. } => {
                enums.insert(name.clone(), variants.clone());
            }
            // Méthodes de trait (V11, DC-3) : enregistrées sous le nom du trait comme « type » de `self`
            // → l'inférence des corps par défaut résout `self.m()`.
            ItemKind::Trait { name, methods } => {
                for m in methods {
                    methods_tbl.insert((name.clone(), m.name.clone()), m.ret.clone());
                }
            }
            ItemKind::Const { .. } => {}
            ItemKind::Impl { type_name, methods, .. } => {
                for m in methods {
                    methods_tbl.insert((type_name.clone(), m.name.clone()), m.ret.clone());
                }
            }
        }
    }

    for (name, ef) in &bindings.fns {
        sigs.insert(name.clone(), (ef.params.clone(), ef.ret.clone()));
    }

    let mut tm = TypeMap::new();
    // Le type de retour effectif (flux) vit dans `tm[fn.id]` : `lower` le lit pour décider de l'emballage.
    for (id, eff) in &fn_effs {
        tm.insert(*id, eff.clone());
    }
    for it in &ast.items {
        if let ItemKind::Fn { params, body, .. } = &it.kind {
            let mut scope: HashMap<String, Type> = params.iter().cloned().collect();
            let mut cx = Inf { sigs: &sigs, structs: &structs, enums: &enums, methods: &methods_tbl, etypes: &bindings.etypes, consts: &bindings.consts, tm: &mut tm };
            for s in body {
                local::stmt(&mut cx, s, &mut scope);
            }
        }
    }

    // Corps des méthodes d'`impl` (self : le type implémenté) ET corps par défaut des traits
    // (self : le trait lui-même, dont les méthodes vivent dans `methods_tbl` sous son nom — DC-3).
    for it in &ast.items {
        let (self_ty, methods) = match &it.kind {
            ItemKind::Impl { type_name, methods, .. } => (type_name.clone(), methods),
            ItemKind::Trait { name, methods } => (name.clone(), methods),
            _ => continue,
        };
        for m in methods {
            if let Some(body) = &m.body {
                let mut scope: HashMap<String, Type> = m.params.iter().cloned().collect();
                if m.self_param {
                    scope.insert("self".to_string(), Type::Struct(self_ty.clone()));
                }
                let mut cx = Inf { sigs: &sigs, structs: &structs, enums: &enums, methods: &methods_tbl, etypes: &bindings.etypes, consts: &bindings.consts, tm: &mut tm };
                for s in body {
                    local::stmt(&mut cx, s, &mut scope);
                }
            }
        }
    }
    tm
}

/// Contexte d'inférence partagé par `local` et `flow`.
pub(crate) struct Inf<'a> {
    pub sigs: &'a Sigs,
    pub structs: &'a Structs,
    pub enums: &'a Enums,
    pub methods: &'a Methods,
    /// Types externes à API (V8, 21100) : nom Iron → `ExternType` (méthodes/ctors).
    pub etypes: &'a HashMap<String, ExternType>,
    /// Constantes externes (V12 [50500]) : `NOM` → (type, chemin Rust).
    pub consts: &'a HashMap<String, (Type, String)>,
    pub tm: &'a mut TypeMap,
}

impl Inf<'_> {
    pub(crate) fn set(&mut self, id: NodeId, ty: Type) -> Type {
        self.tm.insert(id, ty.clone());
        ty
    }
    /// Type de retour d'une méthode externe (V8, 21100) : `obj.m(...)` où `obj : Extern(name)`.
    pub(crate) fn extern_method_ret(&self, iron_name: &str, method: &str) -> Option<Type> {
        let et = self.etypes.get(iron_name)?;
        if let Some((_, f)) = et.methods.iter().find(|(n, _)| n == method) {
            return Some(f.ret.clone());
        }
        // [50600 C1+] fallback : méthode héritée du type de **base** de l'alias (Vector3 → Matrix).
        et.base
            .as_ref()
            .and_then(|b| self.etypes.get(b))
            .and_then(|bet| bet.methods.iter().find(|(n, _)| n == method))
            .map(|(_, f)| f.ret.clone())
    }
    pub(crate) fn field_type(&self, struct_name: &str, field: &str) -> Type {
        self.structs
            .get(struct_name)
            .and_then(|fs| fs.iter().find(|(n, _)| n == field))
            .map(|(_, t)| t.clone())
            .unwrap_or(Type::Unit)
    }

    /// Types des champs liés par un motif de variante, selon le type du scrutateur.
    pub(crate) fn variant_field_types(&self, path: &[String], scrut: &Type) -> Vec<Type> {
        match path {
            [v] => match (v.as_str(), scrut) {
                ("Some", Type::Option(t)) => vec![(**t).clone()],
                ("Ok", Type::Result(t)) => vec![(**t).clone()],
                ("Ok", Type::ResultE(t, _)) => vec![(**t).clone()], // `Result[T, E]` (21500) : `Ok(x)` lie `T`
                ("Err", Type::Result(_)) => vec![Type::Str],
                ("Err", Type::ResultE(_, e)) => vec![(**e).clone()], // `Err(e)` lie `E`
                _ => Vec::new(),
            },
            [en, var] => self
                .enums
                .get(en)
                .and_then(|vs| vs.iter().find(|(n, _)| n == var))
                .map(|(_, f)| f.clone())
                .unwrap_or_default(),
            _ => Vec::new(),
        }
    }

    /// Lie les variables d'un motif dans `scope` (récursif).
    pub(crate) fn bind_pattern(&mut self, p: &Pattern, ty: &Type, scope: &mut HashMap<String, Type>) {
        match p {
            Pattern::Wild | Pattern::Lit(_) => {}
            Pattern::Var(n) => {
                scope.insert(n.clone(), ty.clone());
            }
            Pattern::Variant { path, sub } => {
                let fts = self.variant_field_types(path, ty);
                for (sp, ft) in sub.iter().zip(fts.iter()) {
                    self.bind_pattern(sp, ft, scope);
                }
            }
            Pattern::Tuple(subs) => {
                for (i, sp) in subs.iter().enumerate() {
                    let et = match ty {
                        Type::Tuple(ts) => ts.get(i).cloned().unwrap_or(Type::Unit),
                        _ => Type::Unit,
                    };
                    self.bind_pattern(sp, &et, scope);
                }
            }
            // or-pattern : chaque alternative lie les mêmes variables (mêmes types) → suffit de la 1ʳᵉ.
            Pattern::Or(alts) => {
                if let Some(first) = alts.first() {
                    self.bind_pattern(first, ty, scope);
                }
            }
            // range pattern : aucune variable liée.
            Pattern::Range(..) => {}
        }
    }
}

/// Type d'élément d'un itérable (`for`).
pub(crate) fn element_of(t: &Type) -> Type {
    match t {
        Type::List(e) => (**e).clone(),
        Type::Set(e) => (**e).clone(),
        Type::Dict(k, _) => (**k).clone(), // itère les clés
        Type::Str => Type::Char,           // `for c in s` → c : char (Rust), comme un str d'1 caractère Python
        _ => Type::Unit,
    }
}

// ───────────────────────── Diagnostics de forme (V11, DA-3 / 23100) ─────────────────────────
//
// Au-delà des conflits de primitives (`engine::check_types`), Iron signale **lui-même**, sur la ligne
// Iron et avant `rustc` (filet, I2), trois familles d'erreurs **sur un type concret connu** :
// champ inconnu · méthode inconnue · arité d'appel. **Règle stricte zéro faux-positif** : on ne diagnostique
// QUE lorsque le receveur résout à un `Struct` **Iron connu** (sinon → `rustc`). L'ensemble des méthodes
// d'un struct **inclut celles des traits qu'il implémente** (sinon `dog.shout()` d'une méthode par défaut
// serait un faux-positif).

struct ShapeCx<'a> {
    structs: &'a Structs,
    /// struct → noms de méthodes disponibles (inhérentes + méthodes des traits implémentés).
    struct_methods: &'a HashMap<String, HashSet<String>>,
    /// fn → (args **obligatoires**, args **totaux**) ; `None` par défaut = obligatoire.
    fn_arity: &'a HashMap<String, (usize, usize)>,
    types: &'a TypeMap,
}

/// Diagnostic de forme (DA-3). Construit les tables, puis marche les corps. Première erreur = stop.
pub fn check_shapes(ast: &Ast, types: &TypeMap) -> Result<(), CompileError> {
    let mut structs: Structs = HashMap::new();
    let mut trait_methods: HashMap<String, HashSet<String>> = HashMap::new();
    let mut fn_arity: HashMap<String, (usize, usize)> = HashMap::new();
    for it in &ast.items {
        match &it.kind {
            ItemKind::Struct { name, fields, .. } => {
                structs.insert(name.clone(), fields.clone());
            }
            ItemKind::Trait { name, methods } => {
                trait_methods.insert(name.clone(), methods.iter().map(|m| m.name.clone()).collect());
            }
            ItemKind::Fn { name, params, defaults, .. } => {
                let required = defaults.iter().filter(|d| d.is_none()).count();
                fn_arity.insert(name.clone(), (required, params.len()));
            }
            _ => {}
        }
    }
    // Méthodes par struct : inhérentes + celles de chaque trait implémenté (`impl Trait for S`).
    let mut struct_methods: HashMap<String, HashSet<String>> = HashMap::new();
    for it in &ast.items {
        if let ItemKind::Impl { trait_name, type_name, methods } = &it.kind {
            let set = struct_methods.entry(type_name.clone()).or_default();
            for m in methods {
                set.insert(m.name.clone());
            }
            if let Some(tm) = trait_methods.get(trait_name) {
                set.extend(tm.iter().cloned());
            }
        }
    }
    let cx = ShapeCx { structs: &structs, struct_methods: &struct_methods, fn_arity: &fn_arity, types };
    for it in &ast.items {
        match &it.kind {
            ItemKind::Fn { body, .. } => cx.block(body)?,
            ItemKind::Impl { methods, .. } => {
                for m in methods {
                    if let Some(b) = &m.body {
                        cx.block(b)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

impl ShapeCx<'_> {
    fn block(&self, stmts: &[Stmt]) -> Result<(), CompileError> {
        for s in stmts {
            self.stmt(s)?;
        }
        Ok(())
    }

    fn stmt(&self, s: &Stmt) -> Result<(), CompileError> {
        match &s.kind {
            StmtKind::Let { value, .. } | StmtKind::Yield(value) | StmtKind::ExprStmt(value)
            | StmtKind::Raise { value } => self.expr(value),
            StmtKind::Assign { target, value } => {
                self.expr(target)?;
                self.expr(value)
            }
            StmtKind::Return { value } => value.as_ref().map_or(Ok(()), |e| self.expr(e)),
            StmtKind::If { cond, then, elifs, els } => {
                self.expr(cond)?;
                self.block(then)?;
                for (c, b) in elifs {
                    self.expr(c)?;
                    self.block(b)?;
                }
                self.block(els)
            }
            StmtKind::While { cond, body, els } => {
                self.expr(cond)?;
                self.block(body)?;
                self.block(els)
            }
            StmtKind::For { iter, body, els, .. } => {
                self.expr(iter)?;
                self.block(body)?;
                self.block(els)
            }
            StmtKind::Match { scrutinee, arms } => {
                self.expr(scrutinee)?;
                for a in arms {
                    if let Some(g) = &a.guard {
                        self.expr(g)?;
                    }
                    self.block(&a.body)?;
                }
                Ok(())
            }
            StmtKind::Assert { cond, msg } => {
                self.expr(cond)?;
                msg.as_ref().map_or(Ok(()), |m| self.expr(m))
            }
            StmtKind::Destructure { value, .. } => self.expr(value),
            StmtKind::TryExcept { body, handler, .. } => {
                self.block(body)?;
                self.block(handler)
            }
            StmtKind::Seq(ss) => self.block(ss),
            StmtKind::Break | StmtKind::Continue | StmtKind::Pass | StmtKind::RustBlock(_) => Ok(()),
        }
    }

    fn expr(&self, e: &Expr) -> Result<(), CompileError> {
        match &e.kind {
            ExprKind::Call { callee, args } => {
                // Appel de méthode `base.m(args)` : méthode inconnue sur un struct connu.
                if let ExprKind::Field { base, name } = &callee.kind {
                    self.expr(base)?;
                    if let Some(Type::Struct(s)) = self.types.get(&base.id) {
                        if self.structs.contains_key(s)
                            && !self.struct_methods.get(s).is_some_and(|ms| ms.contains(name))
                        {
                            return Err(CompileError {
                                span: Some(e.span),
                                msg: format!("type : `{s}` n'a pas de méthode `{name}`"),
                            });
                        }
                    }
                } else {
                    // Appel nommé `f(args)` : arité d'une fn connue (hors builtins/HOF, absents de la table).
                    if let ExprKind::Var(f) = &callee.kind {
                        if let Some((req, tot)) = self.fn_arity.get(f) {
                            let n = args.len();
                            if n < *req || n > *tot {
                                return Err(CompileError {
                                    span: Some(e.span),
                                    msg: format!(
                                        "type : `{f}` attend {} argument(s), {n} fourni(s)",
                                        if req == tot { format!("{tot}") } else { format!("{req} à {tot}") }
                                    ),
                                });
                            }
                        }
                    }
                    self.expr(callee)?;
                }
                for a in args {
                    self.expr(a)?;
                }
                Ok(())
            }
            ExprKind::Field { base, name } => {
                self.expr(base)?;
                // champ inconnu sur un struct connu (hors index de tuple `t.0`, hors référence de méthode).
                if name.parse::<usize>().is_err() {
                    if let Some(Type::Struct(s)) = self.types.get(&base.id) {
                        if let Some(fields) = self.structs.get(s) {
                            let is_field = fields.iter().any(|(n, _)| n == name);
                            let is_method = self.struct_methods.get(s).is_some_and(|ms| ms.contains(name));
                            if !is_field && !is_method {
                                return Err(CompileError {
                                    span: Some(e.span),
                                    msg: format!("type : `{s}` n'a pas de champ `{name}`"),
                                });
                            }
                        }
                    }
                }
                Ok(())
            }
            ExprKind::Index { base, idx } => {
                self.expr(base)?;
                self.expr(idx)
            }
            ExprKind::Unary { expr, .. } => self.expr(expr),
            ExprKind::Try { inner } => self.expr(inner),
            ExprKind::Await(inner) => self.expr(inner),
            ExprKind::Binary { lhs, rhs, .. } => {
                self.expr(lhs)?;
                self.expr(rhs)
            }
            ExprKind::StructLit { fields, .. } => {
                for (_, v) in fields {
                    self.expr(v)?;
                }
                Ok(())
            }
            ExprKind::ListLit { items } | ExprKind::Tuple(items) => {
                for it in items {
                    self.expr(it)?;
                }
                Ok(())
            }
            ExprKind::DictLit { pairs } => {
                for (k, v) in pairs {
                    self.expr(k)?;
                    self.expr(v)?;
                }
                Ok(())
            }
            ExprKind::Lambda { body, .. } => self.expr(body),
            ExprKind::Closure { body, .. } => self.block(body),
            ExprKind::Ternary { cond, then, els } => {
                self.expr(cond)?;
                self.expr(then)?;
                self.expr(els)
            }
            ExprKind::Slice { base, lo, hi } => {
                self.expr(base)?;
                if let Some(l) = lo {
                    self.expr(l)?;
                }
                if let Some(h) = hi {
                    self.expr(h)?;
                }
                Ok(())
            }
            ExprKind::Comp { body, iter, cond, .. } => {
                self.expr(body)?;
                self.expr(iter)?;
                if let Some(c) = cond {
                    self.expr(c)?;
                }
                Ok(())
            }
            ExprKind::FString(parts) => {
                for p in parts {
                    if let FStringPart::Expr(ex) = p {
                        self.expr(ex)?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
