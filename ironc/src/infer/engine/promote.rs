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
