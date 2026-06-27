//! [21900] Solveur (V10) : **unification** des contraintes `Eq` (l'overloading `+`/`<`/`==` est résolu par
//! l'unification des opérandes — `s + "!"` unifie `s` à `str`, `i + 1` à `i64`). Les contraintes `Trait`
//! servent au **defaulting** (22100), pas à un rejet ici : `rustc` reste le filet (D7). Un échec
//! d'unification → `TypeError` portant le span Iron (→ 22200).

#![allow(dead_code)]

use super::constraint::Constraint;
use super::subst::Subst;
use crate::ir::{Span, Type};

/// Erreur de type du moteur — message + span(s) Iron (I8).
pub struct TypeError {
    pub msg: String,
    pub span: Span,
}

/// Résout les contraintes `Eq` **au mieux** (mute `subst`) : un conflit est **ignoré** (best-effort, comme
/// l'ancienne pré-passe `params`) — les annotations sont préservées et `rustc` reste le filet (I2/D7). Pour
/// un diagnostic strict (D8), voir `solve_strict`. Les `Trait` sont laissées au defaulting (22100).
pub fn solve(cs: &[Constraint], subst: &mut Subst) {
    for c in cs {
        if let Constraint::Eq(a, b, sp) = c {
            let _ = unify(a.clone(), b.clone(), *sp, subst); // best-effort
        }
    }
}

/// Variante **stricte** : renvoie la **première** erreur d'unification (pour `ironc check` / diagnostics D8).
pub fn solve_strict(cs: &[Constraint], subst: &mut Subst) -> Result<(), TypeError> {
    for c in cs {
        if let Constraint::Eq(a, b, sp) = c {
            unify(a.clone(), b.clone(), *sp, subst)?;
        }
    }
    Ok(())
}

pub fn unify(a: Type, b: Type, sp: Span, subst: &mut Subst) -> Result<(), TypeError> {
    let a = subst.apply(&a);
    let b = subst.apply(&b);
    match (a, b) {
        (x, y) if x == y => Ok(()),
        (Type::Var(v), other) | (other, Type::Var(v)) => {
            if subst.occurs(v, &other) {
                return Err(TypeError { msg: "type récursif (occurs-check)".into(), span: sp });
            }
            subst.bind(v, other);
            Ok(())
        }
        (Type::Option(x), Type::Option(y))
        | (Type::Result(x), Type::Result(y))
        | (Type::List(x), Type::List(y))
        | (Type::Set(x), Type::Set(y))
        | (Type::Shared(x), Type::Shared(y))
        | (Type::SharedSync(x), Type::SharedSync(y))
        | (Type::Weak(x), Type::Weak(y))
        | (Type::Ref(x), Type::Ref(y)) => unify(*x, *y, sp, subst),
        (Type::ResultE(x1, e1), Type::ResultE(x2, e2)) => {
            unify(*x1, *x2, sp, subst)?;
            unify(*e1, *e2, sp, subst)
        }
        (Type::Dict(k1, v1), Type::Dict(k2, v2)) => {
            unify(*k1, *k2, sp, subst)?;
            unify(*v1, *v2, sp, subst)
        }
        (Type::Tuple(xs), Type::Tuple(ys)) if xs.len() == ys.len() => {
            for (x, y) in xs.into_iter().zip(ys) {
                unify(x, y, sp, subst)?;
            }
            Ok(())
        }
        (Type::App(n, xs), Type::App(m, ys)) if n == m && xs.len() == ys.len() => {
            for (x, y) in xs.into_iter().zip(ys) {
                unify(x, y, sp, subst)?;
            }
            Ok(())
        }
        (Type::Fn(ps1, r1), Type::Fn(ps2, r2)) if ps1.len() == ps2.len() => {
            for (x, y) in ps1.into_iter().zip(ps2) {
                unify(x, y, sp, subst)?;
            }
            unify(*r1, *r2, sp, subst)
        }
        // `Result[T]` vs `Result[T, E]` : compat (le moteur ne distingue pas finement ici).
        (Type::Result(x), Type::ResultE(y, _)) | (Type::ResultE(y, _), Type::Result(x)) => {
            unify(*x, *y, sp, subst)
        }
        (x, y) => Err(TypeError {
            msg: format!("types incompatibles : `{}` vs `{}`", show(&x), show(&y)),
            span: sp,
        }),
    }
}

/// Rendu lisible d'un type pour les messages (surface Iron).
pub fn show(t: &Type) -> String {
    match t {
        Type::I64 => "i64".into(),
        Type::F64 => "f64".into(),
        Type::Bool => "bool".into(),
        Type::Str => "str".into(),
        Type::Unit => "unit".into(),
        Type::Char => "char".into(),
        Type::Struct(n) | Type::Enum(n) | Type::Generic(n) => n.clone(),
        Type::Extern(p) => p.clone(),
        Type::Option(x) => format!("Option[{}]", show(x)),
        Type::Result(x) => format!("Result[{}]", show(x)),
        Type::ResultE(x, e) => format!("Result[{}, {}]", show(x), show(e)),
        Type::List(x) => format!("list[{}]", show(x)),
        Type::Set(x) => format!("set[{}]", show(x)),
        Type::Dict(k, v) => format!("dict[{}, {}]", show(k), show(v)),
        Type::Tuple(xs) => format!("({})", xs.iter().map(show).collect::<Vec<_>>().join(", ")),
        Type::Fn(ps, r) => format!("fn({}) -> {}", ps.iter().map(show).collect::<Vec<_>>().join(", "), show(r)),
        Type::Var(n) => format!("?{n}"),
        _ => "…".into(),
    }
}
