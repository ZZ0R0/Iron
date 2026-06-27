# V10 — Réalisation (tracker)

> **État : RÉALISÉ — GATE V10 VERT (14/14).** Le moteur de contraintes remplace l'ancienne heuristique
> `params.rs` (supprimée) ; inférence sans annotation + diagnostics de type natifs. Voir « Honnêteté » plus bas.

## Avancement des pistes

| Piste | Spec | Titre | Vague | Statut |
|---|---|---|---|---|
| 1.1 | 21700 | Fondation : `Type::Var` + substitution + contraintes + structure `engine/` | 1 | ✅ fait |
| 2.1 | 21800 | Génération de contraintes (AST → contraintes, whole-program) | 2 | ✅ fait |
| 2.2 | 21900 | Unification + overloading (`+`/`<`/`==` par contraintes de trait) | 2 | ✅ fait |
| 2.3 | 22000 | Assomptions depuis les bindings + méthodes (couplage auto-binding) | 2 | ✅ fait |
| 3.1 | 22100 | Defaulting + généralisation (let-polymorphism, réécriture AST) | 3 | ✅ fait |
| 3.2 | 22200 | Diagnostics de type Iron natifs (conflit de primitives → span) | 3 | ✅ fait |
| 4.2 | 22300 | Retrait de `params.rs` (subsumé par le moteur) + corpus zéro-annotation | 4 | ✅ fait |
| 4.1 / 4.3 | 22300 | Retrait de `local.rs` / `flow.rs` | 4 | ⏸️ reporté (voir Honnêteté) |
| **5.1** | 22300 | **Gate V10** — zéro-annotation + diagnostics + verdict | 5 | ✅ **VERT 14/14** |

## Honnêteté — ce qui est fait, ce qui est reporté

**Fait (le cœur).** Un **vrai moteur d'inférence par contraintes** (génération → unification union-find →
contraintes de trait pour l'overloading → defaulting → généralisation) remplace l'ancienne pré-passe
**heuristique** `params.rs` (supprimée). Il **lit les bindings** (`query` typé `str` via `levenshtein(str,
str)`), fait de la **vraie polymorphie** (`def id(v): return v` appelé en `i64` ET `str` — impossible
avant), et **diagnostique lui-même** les conflits de primitives sur la ligne Iron, avant `rustc` (I8/D8).
Corpus 50 + binding 30 + vitrine VERTS, **0 unsafe**.

**Reporté (assumé).** Le moteur résout les **types de params/retours** (le « problème d'annotation »).
L'inférence **expression-par-expression** (`local.rs`) et le flux Option/Result (`flow.rs`) restent la base
saine en aval — les **retirer** exigerait que le moteur produise la `TypeMap` **complète** par nœud (chaque
idiome/méthode), une itération plus large qu'on ne risque pas contre les 80 tests verts. C'est donc une
**complétude pragmatique** : le moteur possède l'inférence d'annotations ; `local`/`flow` typent les
expressions. À rouvrir si on veut le moteur seul source de toute la `TypeMap`.

**Limites connues** (annotation encore requise, nouveau « 0.1 % » du typage) : itérer un **param** ne
l'infère pas comme `list[T]` (`for v in xs` → annoter `xs: list[…]`) ; un param **uniquement** en f-string
défaute à `str` même si un appel le voudrait numérique (ambiguïté Display). Documenté, non bloquant.

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
