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

/// **Diagnostic de type Iron** (D8/22200) pour `ironc check` : signale les **conflits entre primitives
/// concrètes** (`str` vs `i64`…) sur la ligne Iron, avant `rustc`. Volontairement conservateur — les
/// conflits structurels (génériques/struct/var) sont laissés au filet `rustc` (D7), pas de faux-positif.
pub fn check_types(ast: &Ast, bindings: &Bindings) -> Result<(), CompileError> {
    let mut subst = subst::Subst::new();
    let env = assume::build_env(ast, bindings, &mut subst);
    let cs = {
        let mut g = gen::Gen::new(&env, &mut subst);
        g.program(ast);
        g.cs
    };
    for c in &cs {
        if let constraint::Constraint::Eq(a, b, sp) = c {
            let (ra, rb) = (subst.apply(a), subst.apply(b));
            if is_primitive(&ra) && is_primitive(&rb) && ra != rb {
                return Err(CompileError {
                    span: Some(*sp),
                    msg: format!("type : `{}` attendu, trouvé `{}`", solve::show(&rb), solve::show(&ra)),
                });
            }
            let _ = solve::unify(a.clone(), b.clone(), *sp, &mut subst); // best-effort pour la suite
        }
    }
    Ok(())
}

fn is_primitive(t: &Type) -> bool {
    matches!(t, Type::I64 | Type::F64 | Type::Str | Type::Bool | Type::Char)
}
