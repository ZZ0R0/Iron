# V11 — Planification (vagues & pistes)

> **Honnêteté du parallélisme** : V11 = beaucoup de features touchant les **fichiers centraux**
> (`parse.rs`/`lower.rs`/`gen.rs`) → **majoritairement séquentiel**. Chaque feature est faite **à fond**
> (parse → IR → lowering → inférence → fixture) avant la suivante. Une seule vraie parallèle (diagnostics,
> `diag.rs` disjoint). 9 pistes · 7 vagues. **Toutes les décisions cadrées sont planifiées** (mapping ci-bas).

## Carte des vagues

| Vague | But | Piste(s) | Parallèle ? |
|---|---|---|---|
| **1 — Fondation** | Geler les formes IR (`Pattern::Or/Range`, bases de classe/traits) | 1.1 | séquentiel |
| **2 — Patterns** | or-patterns + range patterns | 2.1 | séquentiel |
| **3 — Surface** | fn imbriquées · while/for-else · affectation chaînée · const globale | 3.1 | séquentiel |
| **4 — Traits** | `class(Trait)` + bornes user + méthodes par défaut | 4.1 | séquentiel |
| **5 — Dunders** | surcharge d'opérateurs (`__add__`… → `std::ops`) | 5.1 | séquentiel (dép. V4) |
| **6 — Inférence** | enum+anti-unif (6.1) → flux+couverture (6.2) ∥ diagnostics (6.3) | 6.1→6.2 · 6.3 | 6.3 ∥ |
| **7 — Gate** | fixtures de comblement + comparaison-test + verdict | 7.1 | séquentiel |

## Piste ↔ spec ↔ décisions ↔ fichiers possédés

| Piste | Spec | Décisions | Fichier(s) possédé(s) |
|---|---|---|---|
| 1.1 | (toutes) | formes IR | `src/ir.rs` (variantes `Pattern::Or/Range` + `class.traits`) |
| 2.1 | 11000 | DB-1·DB-2·DB-3 | `src/parse.rs` (motifs) · `src/lex.rs` · `src/lower.rs` (bras) |
| 3.1 | 11100 | DD-1·DD-2·DD-3·DD-bonus | `src/parse.rs` (surface) · `src/lower.rs` (désucrage) |
| 4.1 | 23400 | DC-1·DC-2·DC-3 | `src/parse.rs` (bases/trait-corps) · `src/lower.rs` (impl) · `infer/engine/*` (env traits) |
| 5.1 | 23500 | DC-4 | `src/lower.rs` (dunder→impl) · `infer/engine/gen.rs` (op user) |
| 6.1 | 23000 | DA-1·DA-2 | `infer/engine/gen.rs` (enum) · `infer/engine/promote.rs` (anti-unif) |
| 6.2 | 23200 | DA-4·DA-5 | `infer/engine/gen.rs` (flux) · `infer/mod.rs` · **retrait** `infer/flow.rs` |
| 6.3 | 23100 | DA-3 | `infer/engine/diag.rs` · `infer/engine/mod.rs` (check_types) |
| 7.1 | 90900 | hors-périmètre (doc) | `tests/gate-v11.sh` · `tests/fixtures/v11/` · `tests/comparison/` · docs |

## Couverture des décisions (TOUT est planifié)

DA-1→6.1 · DA-2→6.1 · DA-3→6.3 · DA-4→6.2 · DA-5→6.2 · DB-1→2.1 · DB-2→2.1 · DB-3→2.1(rustc, noté) ·
DC-1→4.1 · DC-2→4.1 · DC-3→4.1 · DC-4→5.1 · DD-1→3.1 · DD-2→3.1 · DD-3→3.1 · DD-bonus→3.1 ·
hors-périmètre (global mutable, `dyn`)→7.1(doc). **Aucune décision non planifiée.**

## La fondation (1.1) fige

Les **formes IR** partagées : `Pattern::Or(Vec<Pattern>)`, `Pattern::Range(Box<Expr>, Box<Expr>, bool)`, et
le rattachement **classe → traits implémentés** (champ sur `ItemKind::Struct`/un side-table). Gelé avant le
fan-out pour que parser et lowering aient des cibles stables.

## Invariant dur de réalisation

À chaque vague la colonne **passe l'e2e** (corpus 50 + binding 30 + vitrine + zero-annot). Chaque feature est
**complète** (parse+lower+infer+fixture qui compile+run) avant de clore sa vague — **rien d'à moitié fini**
(exigence utilisateur). Retrait de `flow.rs` (6.2) seulement **prouvé couvert** ; sinon mesuré + documenté.

## Definition of Done — par vague

- **1** : variantes IR ajoutées, `match Type/Pattern` exhaustifs routés ; corpus vert (inerte).
- **2** : `1|2|3`, `0..10`, `0..=10` compilent+matchent ; or-pattern binding incohérent → erreur.
- **3** : fn imbriquée (capture), for/while-else (break vs non), `a=b=expr`, `const` global → fixtures vertes.
- **4** : `class(Trait)` + `def f[T: Trait]` + méthode par défaut → compile+run ; méthode manquante → erreur.
- **5** : `Vec2 + Vec2`, `==`, `<`, `str(v)` via dunders → compile+run ; type sans dunder + `+` → erreur.
- **6** : `area(s)` enum inféré · anti-unif (`count` 2 types) · `flow.rs` retiré (couverture mesurée) ·
  diagnostics champ/méthode/arité/borne natifs.
- **7** : `gate-v11.sh` agrège tout + comparaison-test ; **0 unsafe** ; couverture reportée ; docs à jour ;
  **GATE V11 VERT**.
