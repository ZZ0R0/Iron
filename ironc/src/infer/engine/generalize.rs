//! [22100] Clôture (V10) : **defaulting** des métavariables libres + **réécriture** des types de params/
//! retours non annotés dans l'AST (subsume `params.rs`). Règles de défaut (D5/D6) :
//! var liée → son type · numérique (Add/Sub/Mul/Div/Ord/Eq) → `i64` · affichage (Display) → `str` ·
//! sinon → **générique implicite** (ajouté aux génériques de la fn ; `Clone` par défaut au lowering).

#![allow(dead_code)]

use super::assume::Env;
use super::constraint::{Constraint, TraitBound};
use super::subst::Subst;
use crate::ir::*;
use std::collections::{HashMap, HashSet};

/// Bornes accumulées par métavariable (après application de la subst).
pub fn var_bounds(cs: &[Constraint], subst: &Subst) -> HashMap<u32, HashSet<&'static str>> {
    let mut m: HashMap<u32, HashSet<&'static str>> = HashMap::new();
    for c in cs {
        if let Constraint::Trait(t, b, _) = c {
            if let Type::Var(v) = subst.apply(t) {
                m.entry(v).or_default().insert(b.name());
            }
        }
        let _ = TraitBound::Add; // (garde le type importé even si non listé)
    }
    m
}

/// Décide le type d'une var libre puis le **lie** dans la subst (defaulting global, fait une fois).
/// Les vars sans contrainte exploitable deviennent un générique implicite `__T{v}`.
pub fn default_free_vars(
    cs: &[Constraint],
    subst: &mut Subst,
    used_vars: &HashSet<u32>,
    ret_vars: &HashSet<u32>,
    promoted: &HashSet<u32>,
) {
    let bounds = var_bounds(cs, subst);
    for &v in used_vars {
        if subst.is_bound(v) {
            continue;
        }
        // la var peut être liée transitivement → re-vérifier via apply
        if !matches!(subst.apply(&Type::Var(v)), Type::Var(w) if w == v) {
            continue;
        }
        // Var **promue** (V10.1) : ne PAS défauter — elle deviendra un générique **borné** au writeback.
        if promoted.contains(&v) {
            continue;
        }
        let _ = ret_vars; // (les retours sans valeur ne reçoivent plus de var — voir assume.rs)
        let bs = bounds.get(&v);
        let has = |n: &str| bs.map_or(false, |s| s.contains(n));
        let ty = if has("Add") || has("Sub") || has("Mul") || has("Div") || has("Ord") || has("Eq") {
            Type::I64
        } else if has("Display") {
            Type::Str
        } else {
            Type::Generic(format!("__T{v}")) // générique implicite (pass-through)
        };
        subst.bind(v, ty);
    }
}

/// Résout un type à fond ; toute `Var` résiduelle (jamais défautée) → générique implicite, par sûreté.
fn finalize(t: &Type, subst: &Subst) -> Type {
    let r = subst.apply(t);
    map_vars(&r)
}

fn map_vars(t: &Type) -> Type {
    match t {
        Type::Var(v) => Type::Generic(format!("__T{v}")),
        Type::Option(x) => Type::Option(Box::new(map_vars(x))),
        Type::Result(x) => Type::Result(Box::new(map_vars(x))),
        Type::ResultE(x, e) => Type::ResultE(Box::new(map_vars(x)), Box::new(map_vars(e))),
        Type::List(x) => Type::List(Box::new(map_vars(x))),
        Type::Set(x) => Type::Set(Box::new(map_vars(x))),
        Type::Shared(x) => Type::Shared(Box::new(map_vars(x))),
        Type::SharedSync(x) => Type::SharedSync(Box::new(map_vars(x))),
        Type::Weak(x) => Type::Weak(Box::new(map_vars(x))),
        Type::Ref(x) => Type::Ref(Box::new(map_vars(x))),
        Type::Dict(k, v) => Type::Dict(Box::new(map_vars(k)), Box::new(map_vars(v))),
        Type::Tuple(xs) => Type::Tuple(xs.iter().map(map_vars).collect()),
        Type::App(n, xs) => Type::App(n.clone(), xs.iter().map(map_vars).collect()),
        Type::Fn(ps, r) => Type::Fn(ps.iter().map(map_vars).collect(), Box::new(map_vars(r))),
        _ => t.clone(),
    }
}

/// Collecte les noms de génériques implicites (`__T…`) présents dans un type → à déclarer sur la fn.
fn collect_implicit_generics(t: &Type, out: &mut Vec<String>) {
    match t {
        Type::Generic(g) if g.starts_with("__T") => {
            if !out.contains(g) {
                out.push(g.clone());
            }
        }
        Type::Option(x) | Type::Result(x) | Type::List(x) | Type::Set(x) | Type::Shared(x)
        | Type::SharedSync(x) | Type::Weak(x) | Type::Ref(x) => collect_implicit_generics(x, out),
        Type::ResultE(x, e) => {
            collect_implicit_generics(x, out);
            collect_implicit_generics(e, out);
        }
        Type::Dict(k, v) => {
            collect_implicit_generics(k, out);
            collect_implicit_generics(v, out);
        }
        Type::Tuple(xs) | Type::App(_, xs) => xs.iter().for_each(|x| collect_implicit_generics(x, out)),
        Type::Fn(ps, r) => {
            ps.iter().for_each(|x| collect_implicit_generics(x, out));
            collect_implicit_generics(r, out);
        }
        _ => {}
    }
}

/// Borne de trait (nom interne) → **chemin Rust** pour un générique implicite (V10.1, R2). On ne porte que
/// les bornes qui s'appliquent **proprement** à un générique (`Display`/`PartialOrd`/`PartialEq`) ; les
/// bornes arithmétiques sur un générique (`T + 1`) sont trop fines → laissées au filet `rustc`.
fn rust_bound(name: &str) -> Option<&'static str> {
    match name {
        "Display" => Some("std::fmt::Display"),
        "Ord" => Some("PartialOrd"),
        "Eq" => Some("PartialEq"),
        _ => None,
    }
}

/// Bornes Rust d'un générique implicite `__T{v}` d'après les bornes accumulées de la var `v`.
fn bounds_for(generic: &str, bounds: &HashMap<u32, HashSet<&'static str>>) -> Vec<String> {
    let Some(v) = generic.strip_prefix("__T").and_then(|s| s.parse::<u32>().ok()) else {
        return Vec::new();
    };
    let mut out: Vec<String> = bounds
        .get(&v)
        .map(|bs| bs.iter().filter_map(|b| rust_bound(b)).map(String::from).collect())
        .unwrap_or_default();
    out.sort();
    out.dedup();
    out
}

/// Réécrit dans l'AST les types de params/retours **non annotés** (sentinelle / `Unit`), résolus par le
/// moteur. Ajoute les génériques implicites nécessaires, **bornés** (R2) d'après leurs contraintes.
pub fn writeback(ast: &mut Ast, env: &Env, subst: &Subst, bounds: &HashMap<u32, HashSet<&'static str>>) {
    for it in &mut ast.items {
        match &mut it.kind {
            ItemKind::Fn { name, generics, params, ret, .. } => {
                let Some(sig) = env.fns.get(name) else { continue };
                let mut implicit: Vec<String> = Vec::new();
                for (i, (_, pt)) in params.iter_mut().enumerate() {
                    if is_sentinel(pt) {
                        let resolved = finalize(&sig.params[i], subst);
                        collect_implicit_generics(&resolved, &mut implicit);
                        *pt = resolved;
                    }
                }
                if matches!(ret, Type::Unit) {
                    let resolved = finalize(&sig.ret, subst);
                    // un retour résolu à `Unit` reste `Unit` ; sinon on l'écrit.
                    if !matches!(resolved, Type::Unit) {
                        collect_implicit_generics(&resolved, &mut implicit);
                        *ret = resolved;
                    }
                }
                for g in implicit {
                    if !generics.iter().any(|(n, _)| n == &g) {
                        let b = bounds_for(&g, bounds);
                        generics.push((g, b));
                    }
                }
            }
            // `impl` ET méthodes par défaut de `trait` : réécrit les params/retours non annotés résolus.
            ItemKind::Impl { type_name, methods, .. } | ItemKind::Trait { name: type_name, methods } => {
                for m in methods {
                    let Some(sig) = env.methods.get(&(type_name.clone(), m.name.clone())) else { continue };
                    let off = if sig.has_self { 1 } else { 0 };
                    for (i, (_, pt)) in m.params.iter_mut().enumerate() {
                        if is_sentinel(pt) {
                            *pt = finalize(&sig.params[i + off], subst);
                        }
                    }
                    if matches!(m.ret, Type::Unit) {
                        let resolved = finalize(&sig.ret, subst);
                        if !matches!(resolved, Type::Unit) {
                            m.ret = resolved;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn is_sentinel(t: &Type) -> bool {
    matches!(t, Type::Generic(n) if n == INFER_SENTINEL)
        || matches!(t, Type::Ref(inner) if matches!(&**inner, Type::Generic(n) if n == INFER_SENTINEL))
}
