//! [20100 + 20200] Inférence — réelle (planify-2.3).
//!
//! `local.rs` (monotypes, structs, collections) + `flow.rs` (Option/Result par flux).
//! Best-effort : annote tout `Expr` joignable ; ce que `lower` exige (type des bases d'index,
//! `Option`/`Result`) est couvert. Vérification d'annotations : incrément ultérieur.

mod flow;
mod local;

use crate::ir::*;
use std::collections::HashMap;

/// Signatures de fonctions : nom → (types des params, type de retour **déclaré**).
pub(crate) type Sigs = HashMap<String, (Vec<Type>, Type)>;
/// Champs de structs : nom → (champ → type).
pub(crate) type Structs = HashMap<String, Vec<(String, Type)>>;
/// Enums : nom → variantes (nom de variante → types des champs tuple).
pub(crate) type Enums = HashMap<String, Vec<(String, Vec<Type>)>>;

pub fn infer(ast: &Ast, bindings: &Bindings) -> TypeMap {
    let mut sigs = Sigs::new();
    let mut structs = Structs::new();
    let mut enums = Enums::new();
    for it in &ast.items {
        match &it.kind {
            ItemKind::Fn { name, params, ret, .. } => {
                sigs.insert(name.clone(), (params.iter().map(|(_, t)| t.clone()).collect(), ret.clone()));
            }
            ItemKind::Struct { name, fields, .. } => {
                structs.insert(name.clone(), fields.clone());
            }
            ItemKind::Enum { name, variants, .. } => {
                enums.insert(name.clone(), variants.clone());
            }
            ItemKind::Trait { .. } | ItemKind::Impl { .. } => {}
        }
    }

    for (name, ef) in &bindings.fns {
        sigs.insert(name.clone(), (ef.params.clone(), ef.ret.clone()));
    }

    let mut tm = TypeMap::new();
    for it in &ast.items {
        if let ItemKind::Fn { params, body, .. } = &it.kind {
            let mut scope: HashMap<String, Type> = params.iter().cloned().collect();
            let mut cx = Inf { sigs: &sigs, structs: &structs, enums: &enums, tm: &mut tm };
            // 20200 : flux Option/Result d'abord (calcule le type de retour effectif),
            // puis annotation locale du corps.
            flow::annotate_returns(&mut cx, body);
            for s in body {
                local::stmt(&mut cx, s, &mut scope);
            }
        }
    }

    // Corps des méthodes d'`impl` (self : le type implémenté).
    for it in &ast.items {
        if let ItemKind::Impl { type_name, methods, .. } = &it.kind {
            for m in methods {
                if let Some(body) = &m.body {
                    let mut scope: HashMap<String, Type> = m.params.iter().cloned().collect();
                    if m.self_param {
                        scope.insert("self".to_string(), Type::Struct(type_name.clone()));
                    }
                    let mut cx = Inf { sigs: &sigs, structs: &structs, enums: &enums, tm: &mut tm };
                    for s in body {
                        local::stmt(&mut cx, s, &mut scope);
                    }
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
    pub tm: &'a mut TypeMap,
}

impl Inf<'_> {
    pub(crate) fn set(&mut self, id: NodeId, ty: Type) -> Type {
        self.tm.insert(id, ty.clone());
        ty
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
                ("Err", Type::Result(_)) => vec![Type::Str],
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
        }
    }
}

/// Type d'élément d'un itérable (`for`).
pub(crate) fn element_of(t: &Type) -> Type {
    match t {
        Type::List(e) => (**e).clone(),
        Type::Dict(k, _) => (**k).clone(), // itère les clés
        _ => Type::Unit,
    }
}
