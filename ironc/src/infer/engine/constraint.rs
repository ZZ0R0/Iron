//! [21700] Langage de contraintes (V10) — le **contrat figé** entre génération (21800) et solveur (21900).
//!
//! Deux formes : `Eq` (égalité à unifier) et `Trait` (contrainte de classe, pour l'overloading ad-hoc).
//! **Toute contrainte porte un `Span`** (origine Iron) → c'est ce qui permet à `diag` (22200) de remonter
//! une erreur sur la bonne ligne (I8).

use crate::ir::{Span, Type};

/// Borne de trait **bénie** (D3) : ensemble fixe, mappé **1:1 vers les traits Rust** (I6). Pas de
/// typeclasses générales — ce qui sort de cet ensemble retombe sur le filet `rustc`/annotation (D7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraitBound {
    /// `+` — addition numérique **ou** concaténation de chaînes.
    Add,
    Sub,
    Mul,
    Div,
    /// `<`/`<=`/`>`/`>=` — ordre.
    Ord,
    /// `==`/`!=` — égalité.
    Eq,
    /// f-string / `print` — affichable.
    Display,
}

impl TraitBound {
    pub fn name(self) -> &'static str {
        match self {
            TraitBound::Add => "Add",
            TraitBound::Sub => "Sub",
            TraitBound::Mul => "Mul",
            TraitBound::Div => "Div",
            TraitBound::Ord => "Ord",
            TraitBound::Eq => "Eq",
            TraitBound::Display => "Display",
        }
    }
}

/// Une contrainte de typage, datée d'un `Span` Iron.
#[derive(Debug, Clone)]
pub enum Constraint {
    /// `t1` doit être unifiable avec `t2`.
    Eq(Type, Type, Span),
    /// `t` doit satisfaire la borne `b` (résolu contre les instances connues, 21900).
    Trait(Type, TraitBound, Span),
}

impl Constraint {
    pub fn span(&self) -> Span {
        match self {
            Constraint::Eq(_, _, s) | Constraint::Trait(_, _, s) => *s,
        }
    }
}
