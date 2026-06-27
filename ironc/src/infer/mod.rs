//! [20100 + 20200] Inférence — réelle (planify-2.3).
//!
//! `local.rs` (monotypes, structs, collections) + `flow.rs` (Option/Result par flux).
//! Best-effort : annote tout `Expr` joignable ; ce que `lower` exige (type des bases d'index,
//! `Option`/`Result`) est couvert. Vérification d'annotations : incrément ultérieur.

pub(crate) mod engine; // [V10] moteur d'inférence constraint-based — résout params/retours non annotés
pub(crate) mod flow;
mod local;

use crate::ir::*;
use std::collections::HashMap;

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
                let eff = flow::effective_ret(ret, body);
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
            let mut cx = Inf { sigs: &sigs, structs: &structs, enums: &enums, methods: &methods_tbl, etypes: &bindings.etypes, tm: &mut tm };
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
                let mut cx = Inf { sigs: &sigs, structs: &structs, enums: &enums, methods: &methods_tbl, etypes: &bindings.etypes, tm: &mut tm };
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
    pub tm: &'a mut TypeMap,
}

impl Inf<'_> {
    pub(crate) fn set(&mut self, id: NodeId, ty: Type) -> Type {
        self.tm.insert(id, ty.clone());
        ty
    }
    /// Type de retour d'une méthode externe (V8, 21100) : `obj.m(...)` où `obj : Extern(name)`.
    pub(crate) fn extern_method_ret(&self, iron_name: &str, method: &str) -> Option<Type> {
        self.etypes
            .get(iron_name)
            .and_then(|et| et.methods.iter().find(|(n, _)| n == method))
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
        _ => Type::Unit,
    }
}
