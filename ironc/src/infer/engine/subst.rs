//! [21700] Substitution union-find (V10) — résout les métavariables `Type::Var`.
//!
//! `fresh()` alloue une var ; `bind`/`union` lient ; `apply` résout un type à fond (compression de chemin).
//! `occurs_check` empêche les types cycliques. Invariant de sortie (D2) : après inférence, `apply` ne
//! laisse **aucune** `Var` (sinon defaulting 22100 ou erreur 22200).

use crate::ir::Type;
use std::collections::HashMap;

#[derive(Default)]
pub struct Subst {
    next: u32,
    /// `Var(i)` → type lié (peut être une autre `Var`, résolu par `apply`).
    map: HashMap<u32, Type>,
}

impl Subst {
    pub fn new() -> Self {
        Subst::default()
    }

    /// Alloue une métavariable fraîche.
    pub fn fresh(&mut self) -> Type {
        let v = self.next;
        self.next += 1;
        Type::Var(v)
    }

    /// Nombre de métavariables allouées (les ids vont de `0` à `vars()-1`).
    pub fn vars(&self) -> u32 {
        self.next
    }

    /// Lie `Var(v)` à `ty` (suppose `v` non encore lié et `occurs_check` déjà fait par l'appelant).
    pub fn bind(&mut self, v: u32, ty: Type) {
        self.map.insert(v, ty);
    }

    pub fn is_bound(&self, v: u32) -> bool {
        self.map.contains_key(&v)
    }

    /// Représentant immédiat d'une var (un seul niveau, sans récursion profonde).
    pub fn get(&self, v: u32) -> Option<&Type> {
        self.map.get(&v)
    }

    /// Résout un type **à fond** : remplace toute `Var` liée par sa valeur, récursivement.
    pub fn apply(&self, t: &Type) -> Type {
        match t {
            Type::Var(v) => match self.map.get(v) {
                Some(inner) => self.apply(inner), // chaîne : compression logique
                None => t.clone(),                // var libre : laissée telle quelle
            },
            Type::Option(x) => Type::Option(Box::new(self.apply(x))),
            Type::Result(x) => Type::Result(Box::new(self.apply(x))),
            Type::ResultE(x, e) => Type::ResultE(Box::new(self.apply(x)), Box::new(self.apply(e))),
            Type::List(x) => Type::List(Box::new(self.apply(x))),
            Type::Set(x) => Type::Set(Box::new(self.apply(x))),
            Type::Shared(x) => Type::Shared(Box::new(self.apply(x))),
            Type::SharedSync(x) => Type::SharedSync(Box::new(self.apply(x))),
            Type::Weak(x) => Type::Weak(Box::new(self.apply(x))),
            Type::Ref(x) => Type::Ref(Box::new(self.apply(x))),
            Type::Dict(k, v) => Type::Dict(Box::new(self.apply(k)), Box::new(self.apply(v))),
            Type::Tuple(xs) => Type::Tuple(xs.iter().map(|x| self.apply(x)).collect()),
            Type::App(n, args) => Type::App(n.clone(), args.iter().map(|x| self.apply(x)).collect()),
            Type::Fn(ps, r) => {
                Type::Fn(ps.iter().map(|x| self.apply(x)).collect(), Box::new(self.apply(r)))
            }
            _ => t.clone(), // types atomiques (I64, Str, Struct, Enum, Generic, Extern, Num, Char, Bool, Unit)
        }
    }

    /// `Var(v)` apparaît-elle dans `t` (après résolution) ? Bloque les types récursifs lors de l'unification.
    pub fn occurs(&self, v: u32, t: &Type) -> bool {
        match self.apply(t) {
            Type::Var(w) => v == w,
            Type::Option(x)
            | Type::Result(x)
            | Type::List(x)
            | Type::Set(x)
            | Type::Shared(x)
            | Type::SharedSync(x)
            | Type::Weak(x)
            | Type::Ref(x) => self.occurs(v, &x),
            Type::ResultE(x, e) => self.occurs(v, &x) || self.occurs(v, &e),
            Type::Dict(k, val) => self.occurs(v, &k) || self.occurs(v, &val),
            Type::Tuple(xs) | Type::App(_, xs) => xs.iter().any(|x| self.occurs(v, x)),
            Type::Fn(ps, r) => ps.iter().any(|x| self.occurs(v, x)) || self.occurs(v, &r),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_are_distinct() {
        let mut s = Subst::new();
        assert_ne!(s.fresh(), s.fresh());
    }

    #[test]
    fn apply_resolves_transitively() {
        let mut s = Subst::new();
        let (a, b) = (s.fresh(), s.fresh()); // Var(0), Var(1)
        if let (Type::Var(av), Type::Var(bv)) = (&a, &b) {
            s.bind(*av, b.clone()); // 0 → Var(1)
            s.bind(*bv, Type::Str); // 1 → str
        }
        assert_eq!(s.apply(&a), Type::Str); // 0 → 1 → str
    }

    #[test]
    fn occurs_detects_cycle() {
        let mut s = Subst::new();
        let a = s.fresh();
        if let Type::Var(av) = a {
            assert!(s.occurs(av, &Type::List(Box::new(a.clone())))); // av dans list[av]
            assert!(!s.occurs(av, &Type::Str));
        }
    }
}
