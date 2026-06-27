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
pub mod flux; // [V11/DA-4] retour effectif Option/Result par le flux (ex-`infer/flow.rs`)
mod gen;
mod generalize;
mod promote;
mod solve;

use crate::ir::*;
use std::collections::HashSet;

/// **Résolution des types de params/retours non annotés** par le moteur de contraintes (subsume
/// `params.rs`). Mute l'AST (remplit les sentinelles) ; `local`/`flow`/`lower` tournent ensuite inchangés.
/// Pipeline : `build_env` → `gen` (contraintes) → `solve` (unification) → defaulting → `writeback`.
pub fn resolve_params(ast: &mut Ast, bindings: &Bindings) -> Result<(), CompileError> {
    // `build_env` alloue les vars de params/retours une fois → ids `< boundary` **stables** d'un tour à
    // l'autre. Chaque tour repart d'une `Subst` neuve (`with_next(boundary)`), seule la promotion persiste.
    let mut counter = subst::Subst::new();
    let mut env = assume::build_env(ast, bindings, &mut counter);
    let pre_boundary = counter.vars();

    let mut promoted = promote::Promoted::new();

    // Pré-passe DA-2 (anti-unification) : avant de figer `boundary`, on collecte par var de param ses types
    // d'appel et on anti-unifie. Deux instances structurellement semblables divergeant à une position
    // (`list[i64]`/`list[str]`) → la **shape** `list[T]` est injectée dans l'env (la var `T` promue) ; le
    // param n'est plus promu entier mais **généralisé à la position**. Conflit à plat (`i64`/`str`) → shape
    // = var nue → laissé à la promotion-entière de la boucle.
    {
        let mut pre = subst::Subst::with_next(pre_boundary);
        let pre_cs = {
            let mut g = gen::Gen::new(&env, &mut pre, &promoted);
            g.program(ast);
            g.cs
        };
        let _ = promote::solve_and_detect(&pre_cs, &mut pre, pre_boundary);
        let mut inst: std::collections::HashMap<u32, Vec<Type>> = std::collections::HashMap::new();
        for c in &pre_cs {
            if let constraint::Constraint::Eq(a, b, _) = c {
                if let Type::Var(v) = a {
                    if *v < pre_boundary {
                        inst.entry(*v).or_default().push(pre.apply(b));
                    }
                }
                if let Type::Var(v) = b {
                    if *v < pre_boundary {
                        inst.entry(*v).or_default().push(pre.apply(a));
                    }
                }
            }
        }
        for (v, types) in inst {
            let mut distinct: Vec<Type> = Vec::new();
            for t in types {
                if !distinct.contains(&t) {
                    distinct.push(t);
                }
            }
            if distinct.len() < 2 {
                continue;
            }
            let mut fresh = Vec::new();
            let mut shape = distinct[0].clone();
            for t in &distinct[1..] {
                shape = promote::anti_unify(&shape, t, &mut counter, &mut fresh);
            }
            // structuré (constructeur partagé + ≥1 position généralisée) → injecter ; sinon promotion entière.
            if !matches!(shape, Type::Var(_)) && !fresh.is_empty() {
                env.replace_param_var(v, &shape);
                promoted.extend(fresh);
            }
        }
    }

    let boundary = counter.vars();

    // Boucle de point fixe (V10.1, D-g/D-a) : génère → unifie → détecte les conflits de params → promeut →
    // recommence, jusqu'à stabilité. Bornée (garde-fou anti-boucle ; converge car la promotion ne fait que
    // relâcher). En vague 1 : `gen` freshise tout (V10) → aucun conflit → un seul tour.
    let (mut subst, mut cs);
    let mut guard = 0;
    loop {
        subst = subst::Subst::with_next(boundary);
        cs = {
            let mut g = gen::Gen::new(&env, &mut subst, &promoted);
            g.program(ast);
            g.cs
        };
        let conflicts = promote::solve_and_detect(&cs, &mut subst, boundary);
        let fresh_conflicts: Vec<u32> = conflicts.into_iter().filter(|v| !promoted.contains(v)).collect();
        guard += 1;
        if fresh_conflicts.is_empty() || guard > 1000 {
            break;
        }
        promoted.extend(fresh_conflicts);
    }

    // Résolution structurelle (V10.1, D-g) : on **re-génère en gardant la `Subst`** (params déjà liés par
    // les appels). Les accès `x.name` / `obj.m()` / `xs[i]` qui s'étaient « abstenus » au 1ᵉʳ tour (base
    // inconnue) se résolvent maintenant ; un `return x.name` devient concret. Point fixe : tant que de
    // nouvelles vars de params/retours se lient.
    let mut sguard = 0;
    loop {
        let before = subst.bound_below(boundary);
        cs = {
            let mut g = gen::Gen::new(&env, &mut subst, &promoted);
            g.program(ast);
            g.cs
        };
        promote::solve_and_detect(&cs, &mut subst, boundary);
        sguard += 1;
        if subst.bound_below(boundary) == before || sguard > 20 {
            break;
        }
    }

    // Defaulting **en dernier** (D-j), après convergence. Les vars promues restent libres → génériques
    // **bornés** (R2) au writeback ; les autres libres sont défautées (num→i64, Display→str, sinon générique).
    let bounds = generalize::var_bounds(&cs, &subst);
    let used: HashSet<u32> = (0..subst.vars()).collect();
    generalize::default_free_vars(&cs, &mut subst, &used, &env.ret_vars, &promoted);
    generalize::writeback(ast, &env, &subst, &bounds);
    Ok(())
}

/// **Diagnostic de type Iron** (D8/22200) pour `ironc check` : signale les **conflits entre primitives
/// concrètes** (`str` vs `i64`…) sur la ligne Iron, avant `rustc`. Volontairement conservateur — les
/// conflits structurels (génériques/struct/var) sont laissés au filet `rustc` (D7), pas de faux-positif.
pub fn check_types(ast: &Ast, bindings: &Bindings) -> Result<(), CompileError> {
    let mut subst = subst::Subst::new();
    let env = assume::build_env(ast, bindings, &mut subst);
    let empty = promote::Promoted::new();
    let cs = {
        let mut g = gen::Gen::new(&env, &mut subst, &empty);
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

/// **Couverture du moteur** (DA-5) — mesurée sur l'AST **après** `resolve_params`. Compte les emplacements
/// de **signature** (params + retour de valeur) de chaque fn/méthode et combien sont **résolus concrets**
/// (le moteur a fait son travail : ni sentinelle d'inférence, ni `Var` résiduelle). Sur du code **zéro
/// annotation**, un ratio 100 % prouve que le moteur type seul toutes les signatures.
///
/// **Reliquat `local.rs` (documenté, DA-5)** : `local.rs` n'est **pas** retiré car il produit la `TypeMap`
/// **par expression** que `lower` consomme (type de *chaque* nœud), là où le moteur ne résout que les
/// **signatures**. Les deux couches sont **complémentaires**, pas redondantes — retrait à l'aveugle refusé.
pub fn coverage(ast: &Ast) -> (usize, usize) {
    let mut total = 0usize;
    let mut resolved = 0usize;
    let mut tally = |t: &Type| {
        total += 1;
        if is_resolved(t) {
            resolved += 1;
        }
    };
    for it in &ast.items {
        match &it.kind {
            ItemKind::Fn { params, ret, body, .. } => {
                for (_, t) in params {
                    tally(t);
                }
                if assume::has_value_return(body) {
                    tally(ret);
                }
            }
            ItemKind::Impl { methods, .. } => {
                for m in methods {
                    for (_, t) in &m.params {
                        tally(t);
                    }
                    if m.body.as_deref().is_some_and(assume::has_value_return) {
                        tally(&m.ret);
                    }
                }
            }
            _ => {}
        }
    }
    (resolved, total)
}

/// Type de signature **résolu concret** ? (ni sentinelle d'inférence non comblée, ni métavariable résiduelle).
/// Un `Generic` issu d'une promotion compte comme résolu (le moteur a tranché : polymorphe).
fn is_resolved(t: &Type) -> bool {
    match t {
        Type::Var(_) => false,
        Type::Generic(n) => n != crate::ir::INFER_SENTINEL,
        Type::Option(x) | Type::Result(x) | Type::List(x) | Type::Set(x) | Type::Shared(x)
        | Type::SharedSync(x) | Type::Weak(x) | Type::Ref(x) => is_resolved(x),
        Type::ResultE(x, e) => is_resolved(x) && is_resolved(e),
        Type::Dict(k, v) => is_resolved(k) && is_resolved(v),
        Type::Tuple(xs) | Type::App(_, xs) => xs.iter().all(is_resolved),
        Type::Fn(ps, r) => ps.iter().all(is_resolved) && is_resolved(r),
        _ => true,
    }
}
