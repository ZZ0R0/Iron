# V10 — Réalisation (tracker)

> **État : cadrage + planification FAITS. Réalisation NON commencée** (attend le feu vert). Liste propre,
> 1:1 avec les pistes. 10 pistes · 5 vagues · 7 specs.

## Avancement des pistes

| Piste | Spec | Titre | Vague | Parallèle | Statut |
|---|---|---|---|---|---|
| 1.1 | 21700 | Fondation : vars + substitution + contraintes + structure `engine/` | 1 | séquentiel | ☐ todo |
| 2.1 | 21800 | Génération de contraintes (AST → contraintes) | 2 | ∥ | ☐ todo |
| 2.2 | 21900 | Unification + résolution des contraintes de trait (overloading) | 2 | ∥ | ☐ todo |
| 2.3 | 22000 | Assomptions depuis les bindings + méthodes | 2 | ∥ | ☐ todo |
| 3.1 | 22100 | Defaulting + généralisation (let-polymorphism) | 3 | ∥ | ☐ todo |
| 3.2 | 22200 | Diagnostics de type Iron (mismatch → span) | 3 | ∥ | ☐ todo |
| 4.1 | 22300 | Migration — brancher le moteur, retirer `local.rs` | 4 | séquentiel | ☐ todo |
| 4.2 | 22300 | Migration — retirer `params.rs`, corpus zéro-annotation | 4 | séquentiel | ☐ todo |
| 4.3 | 22300 | Migration — I11 en contraintes, retirer `flow.rs` | 4 | séquentiel | ☐ todo |
| **5.1** | 22300 | **Gate V10** — zéro-annotation + audit + verdict | 5 | séquentiel | ☐ todo |

## Décisions figées (rappel — voir README cadrage)

D0 complète-pragmatique · D1 contraintes+unification bidirectionnelle · D2 `Type::Var` (zéro `Var` en
sortie) · D3 overloading par contraintes de trait · D4 bindings = assomptions · D5 defaulting · D6
généralisation `[T]` · D7 `rustc` filet (I2) · D8 erreurs de type Iron (I8) · D9 migration sans big-bang.

## Invariant dur de réalisation

L'ancienne inférence reste branchée jusqu'à la vague 4 ; la colonne (corpus 50 + binding 30 + vitrine) passe
l'e2e **en permanence**. On ne retire `local`/`params`/`flow` que quand le moteur couvre leur rôle, prouvé
par le corpus. Une bascule qui casse l'e2e → ré-unifier avant d'avancer.

## Écarts / faiblesses (à remplir en réalisation)

- (rien encore)
