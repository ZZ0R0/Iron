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
use std::collections::HashSet;

/// **Résolution des types de params/retours non annotés** par le moteur de contraintes (subsume
/// `params.rs`). Mute l'AST (remplit les sentinelles) ; `local`/`flow`/`lower` tournent ensuite inchangés.
/// Pipeline : `build_env` → `gen` (contraintes) → `solve` (unification) → defaulting → `writeback`.
pub fn resolve_params(ast: &mut Ast, bindings: &Bindings) -> Result<(), CompileError> {
    let mut subst = subst::Subst::new();
    let env = assume::build_env(ast, bindings, &mut subst);

    let cs = {
        let mut g = gen::Gen::new(&env, &mut subst);
        g.program(ast);
        g.cs
    };

    solve::solve(&cs, &mut subst); // best-effort (rustc reste le filet, D7) ; diagnostics stricts = `check`

    let used: HashSet<u32> = (0..subst.vars()).collect();
    generalize::default_free_vars(&cs, &mut subst, &used, &env.ret_vars);
    generalize::writeback(ast, &env, &subst);
    Ok(())
}
