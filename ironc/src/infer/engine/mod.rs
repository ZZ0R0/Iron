//! [21700] Moteur d'inférence V10 — **constraint-based, bidirectionnel**.
//!
//! Remplace (progressivement, D9) le faisceau d'heuristiques `local`/`params`/`flow`. Pipeline interne :
//! `gen` (AST → contraintes) → `solve` (unification + traits) → `generalize` (defaulting + let-poly) →
//! `TypeMap` complète, `diag` traduisant les erreurs sur la ligne Iron (I8). `assume` ingère les bindings
//! `.iri` (couplage auto-binding, D4).
//!
//! **Vague 1** : socle (`subst`, `constraint`) + structure. L'entrée `infer_types` est un **stub non
//! branché** — l'ancienne inférence pilote encore le pipeline. Les vagues 2-4 remplissent et branchent.
#![allow(dead_code)] // en construction (vagues 2-4) ; branché à la migration

pub mod constraint;
pub mod subst;

mod assume;
mod diag;
mod gen;
mod generalize;
mod solve;

use crate::ir::*;
use std::collections::HashMap;

/// Entrée du moteur (D1) : AST + bindings → `TypeMap` complète, ou erreur de type Iron (D8).
/// Stub en vague 1 (non appelé) ; orchestré en vague 4.
#[allow(dead_code)]
pub fn infer_types(ast: &Ast, bindings: &Bindings) -> Result<TypeMap, CompileError> {
    let _ = (ast, bindings);
    Ok(HashMap::new())
}
