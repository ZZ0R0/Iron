//! [22000] Environnement de typage (V10) : signatures de fonctions/méthodes/structs/enums + **assomptions
//! depuis les bindings** (D4 — couplage auto-binding). Les params/retours **non annotés** reçoivent une
//! métavariable fraîche (allouée dans le `Subst` partagé) ; les annotés gardent leur type. C'est la base
//! que `gen` consomme pour émettre les contraintes.
#![allow(dead_code)]

use super::subst::Subst;
use crate::ir::*;
use std::collections::{HashMap, HashSet};

/// Signature d'une fonction : types de params (var si non annoté), retour (var si non annoté), génériques.
#[derive(Clone)]
pub struct FnSig {
    pub params: Vec<Type>,
    pub ret: Type,
    pub generics: Vec<String>,
    /// `self` implicite (méthode) en tête de `params` ? (le receveur est `params[0]`).
    pub has_self: bool,
}

#[derive(Default)]
pub struct Env {
    pub fns: HashMap<String, FnSig>,
    pub methods: HashMap<(String, String), FnSig>, // (type receveur, méthode) — Iron impls + bindings
    pub ctors: HashMap<(String, String), FnSig>,   // (type, ctor externe)
    pub structs: HashMap<String, Vec<(String, Type)>>,
    pub enums: HashMap<String, Vec<(String, Vec<Type>)>>,
    pub extern_fns: HashMap<String, (Vec<Type>, Type)>, // bindings.fns sans receveur
    pub extern_types: Vec<String>,                      // noms de types externes connus
    /// Métavariables qui sont des **retours** non annotés : si elles restent libres (aucun `return
    /// <valeur>` ne les contraint), elles défautent à `Unit` (pas un générique).
    pub ret_vars: HashSet<u32>,
}

/// Type non annoté ? (sentinelle param) → métavariable fraîche. Sinon → garde le type déclaré.
fn var_if_infer(t: &Type, subst: &mut Subst) -> Type {
    match t {
        Type::Generic(n) if n == INFER_SENTINEL => subst.fresh(),
        Type::Ref(inner) if matches!(&**inner, Type::Generic(n) if n == INFER_SENTINEL) => {
            Type::Ref(Box::new(subst.fresh()))
        }
        _ => t.clone(),
    }
}

/// Retour non annoté (`Unit`) **et** la fn renvoie une valeur → métavariable fraîche. Sinon (annoté, ou
/// aucune valeur retournée → vraie fn `Unit`) → type déclaré. Évite de génériciser un `Unit` (`main`).
fn ret_var_if_infer(t: &Type, body: &[Stmt], subst: &mut Subst) -> Type {
    match t {
        Type::Unit if has_value_return(body) => subst.fresh(),
        _ => t.clone(),
    }
}

/// La fn renvoie-t-elle une valeur quelque part (`return <expr>`) ?
fn has_value_return(body: &[Stmt]) -> bool {
    body.iter().any(|s| match &s.kind {
        StmtKind::Return { value } => value.is_some(),
        StmtKind::If { then, elifs, els, .. } => {
            has_value_return(then) || elifs.iter().any(|(_, b)| has_value_return(b)) || has_value_return(els)
        }
        StmtKind::While { body, .. } | StmtKind::For { body, .. } => has_value_return(body),
        StmtKind::Match { arms, .. } => arms.iter().any(|a| has_value_return(&a.body)),
        StmtKind::TryExcept { body, handler, .. } => has_value_return(body) || has_value_return(handler),
        _ => false,
    })
}

pub fn build_env(ast: &Ast, bindings: &Bindings, subst: &mut Subst) -> Env {
    let mut env = Env::default();

    for it in &ast.items {
        match &it.kind {
            ItemKind::Struct { name, fields, .. } => {
                env.structs.insert(name.clone(), fields.clone());
            }
            ItemKind::Enum { name, variants, .. } => {
                env.enums.insert(name.clone(), variants.clone());
            }
            _ => {}
        }
    }

    for it in &ast.items {
        if let ItemKind::Fn { name, generics, params, ret, body, .. } = &it.kind {
            let psig = params.iter().map(|(_, t)| var_if_infer(t, subst)).collect();
            let rsig = ret_var_if_infer(ret, body, subst);
            if let Type::Var(id) = &rsig {
                env.ret_vars.insert(*id);
            }
            env.fns.insert(
                name.clone(),
                FnSig {
                    params: psig,
                    ret: rsig,
                    generics: generics.iter().map(|(g, _)| g.clone()).collect(),
                    has_self: false,
                },
            );
        }
    }

    for it in &ast.items {
        if let ItemKind::Impl { type_name, methods, .. } = &it.kind {
            for m in methods {
                let mut ps: Vec<Type> = Vec::new();
                if m.self_param {
                    ps.push(Type::Struct(type_name.clone()));
                }
                ps.extend(m.params.iter().map(|(_, t)| var_if_infer(t, subst)));
                let rsig = ret_var_if_infer(&m.ret, m.body.as_deref().unwrap_or(&[]), subst);
                if let Type::Var(id) = &rsig {
                    env.ret_vars.insert(*id);
                }
                env.methods.insert(
                    (type_name.clone(), m.name.clone()),
                    FnSig { params: ps, ret: rsig, generics: Vec::new(), has_self: m.self_param },
                );
            }
        }
    }

    for (name, ef) in &bindings.fns {
        if ef.recv.is_none() {
            env.extern_fns.insert(name.clone(), (ef.params.clone(), ef.ret.clone()));
        }
    }
    for (tname, et) in &bindings.etypes {
        env.extern_types.push(tname.clone());
        for (mname, ef) in &et.methods {
            let mut ps = vec![Type::Struct(tname.clone())];
            ps.extend(ef.params.clone());
            env.methods.insert(
                (tname.clone(), mname.clone()),
                FnSig { params: ps, ret: ef.ret.clone(), generics: Vec::new(), has_self: true },
            );
        }
        for (cname, ef) in &et.ctors {
            env.ctors.insert(
                (tname.clone(), cname.clone()),
                FnSig { params: ef.params.clone(), ret: ef.ret.clone(), generics: Vec::new(), has_self: false },
            );
        }
    }
    for tname in bindings.types.keys() {
        env.extern_types.push(tname.clone());
    }

    env
}
