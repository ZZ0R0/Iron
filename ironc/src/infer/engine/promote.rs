//! [V10.1 / R1] Promotion sur conflit — le cœur du modèle « piloté par les sites d'appel ».
//!
//! Par défaut un param non annoté est **monomorphe** : ses args d'appel s'unifient avec sa **vraie** var.
//! Si deux sites d'appel lui donnent des types **incompatibles**, on le **promeut générique** (D-a) : son id
//! entre dans `Promoted`, et `gen` le **freshise** à chaque appel (let-polymorphism). La boucle de point
//! fixe (mod.rs) re-génère jusqu'à stabilité.
//!
//! Les ids `< boundary` sont les vars de params/retours (stables, alloués une fois). Une var de param y figure
//! ⟺ elle a été promue.

use super::constraint::Constraint;
use super::subst::Subst;
use crate::ir::Type;
use std::collections::HashSet;

/// Ensemble des **vars de params promues en génériques** (ids stables `< boundary`).
pub type Promoted = HashSet<u32>;

/// Unifie best-effort **et** collecte en un seul passage les vars de **params non annotés** (`id <
/// boundary`) sur-contraintes : un `Eq` qui **échoue** et dont un côté original est une var de param ⇒ ce
/// param reçoit deux types incompatibles ⇒ candidat à la promotion. (Un param **annoté** n'est pas une var
/// → traité par `check_types`/`rustc`, D-b.) Remplace `solve::solve` dans la boucle de point fixe.
pub fn solve_and_detect(cs: &[Constraint], subst: &mut Subst, boundary: u32) -> HashSet<u32> {
    let mut conflicts = HashSet::new();
    for c in cs {
        if let Constraint::Eq(a, b, sp) = c {
            if super::solve::unify(a.clone(), b.clone(), *sp, subst).is_err() {
                for v in param_vars_of(a, boundary).into_iter().chain(param_vars_of(b, boundary)) {
                    conflicts.insert(v);
                }
            }
        }
    }
    conflicts
}

/// Anti-unification (LGG, DA-2) de deux types : positions **communes** conservées, positions **divergentes**
/// → une var fraîche (allouée via `subst`, id poussé dans `fresh`). Constructeurs différents (ou arités
/// différentes) → généralisation **totale** à cette position (var fraîche). Best-effort, structurel.
/// `list[i64]` ∧ `list[str]` = `list[T]` ; `i64` ∧ `str` = `T` (bare) ; `list[i64]` ∧ `list[i64]` = `list[i64]`.
pub fn anti_unify(a: &Type, b: &Type, subst: &mut Subst, fresh: &mut Vec<u32>) -> Type {
    use Type::*;
    if a == b {
        return a.clone();
    }
    let au = |x: &Type, y: &Type, subst: &mut Subst, fresh: &mut Vec<u32>| {
        Box::new(anti_unify(x, y, subst, fresh))
    };
    match (a, b) {
        (List(x), List(y)) => List(au(x, y, subst, fresh)),
        (Set(x), Set(y)) => Set(au(x, y, subst, fresh)),
        (Option(x), Option(y)) => Option(au(x, y, subst, fresh)),
        (Result(x), Result(y)) => Result(au(x, y, subst, fresh)),
        (Ref(x), Ref(y)) => Ref(au(x, y, subst, fresh)),
        (Shared(x), Shared(y)) => Shared(au(x, y, subst, fresh)),
        (SharedSync(x), SharedSync(y)) => SharedSync(au(x, y, subst, fresh)),
        (Weak(x), Weak(y)) => Weak(au(x, y, subst, fresh)),
        (ResultE(x1, e1), ResultE(x2, e2)) => {
            let t = au(x1, x2, subst, fresh);
            let e = au(e1, e2, subst, fresh);
            ResultE(t, e)
        }
        (Dict(k1, v1), Dict(k2, v2)) => {
            let k = au(k1, k2, subst, fresh);
            let v = au(v1, v2, subst, fresh);
            Dict(k, v)
        }
        (Tuple(xs), Tuple(ys)) if xs.len() == ys.len() => {
            Tuple(xs.iter().zip(ys).map(|(x, y)| anti_unify(x, y, subst, fresh)).collect())
        }
        (App(n1, xs), App(n2, ys)) if n1 == n2 && xs.len() == ys.len() => {
            App(n1.clone(), xs.iter().zip(ys).map(|(x, y)| anti_unify(x, y, subst, fresh)).collect())
        }
        // constructeurs incompatibles → position entièrement généralisée.
        _ => {
            let f = subst.fresh();
            if let Type::Var(id) = f {
                fresh.push(id);
            }
            f
        }
    }
}

/// Vars de params (`id < boundary`) apparaissant **syntaxiquement** dans un type (avant résolution).
fn param_vars_of(t: &Type, boundary: u32) -> Vec<u32> {
    let mut out = Vec::new();
    collect(t, boundary, &mut out);
    out
}

fn collect(t: &Type, boundary: u32, out: &mut Vec<u32>) {
    match t {
        Type::Var(v) if *v < boundary => out.push(*v),
        Type::Var(_) => {}
        Type::Option(x) | Type::Result(x) | Type::List(x) | Type::Set(x) | Type::Shared(x)
        | Type::SharedSync(x) | Type::Weak(x) | Type::Ref(x) => collect(x, boundary, out),
        Type::ResultE(x, e) => {
            collect(x, boundary, out);
            collect(e, boundary, out);
        }
        Type::Dict(k, v) => {
            collect(k, boundary, out);
            collect(v, boundary, out);
        }
        Type::Tuple(xs) | Type::App(_, xs) => xs.iter().for_each(|x| collect(x, boundary, out)),
        Type::Fn(ps, r) => {
            ps.iter().for_each(|x| collect(x, boundary, out));
            collect(r, boundary, out);
        }
        _ => {}
    }
}
