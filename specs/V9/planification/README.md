# V9 — Planification (vagues & pistes)

> Version de **clôture** : 6 vagues séquentielles, 22 pistes. La 1ʳᵉ vague **gèle les ajouts IR**
> (`SharedSync`, `async`/`await`, `yield`, tags de protocole) → les vagues suivantes remplissent sans
> re-toucher `ir.rs` (anti-collision V6/V7/V8). Gabarit : `generator §6`. Parente : `../README.md`.

## Table de correspondance piste → spec

| Piste | Spec | Titre | Vague |
|---|---|---|---|
| **1.1** | — | **Fondation** : gel IR (`SharedSync`, `async`/`await`, `Yield`, tags proto) + arms défaut | 1 |
| 2.1 | 51600 | **⭐ Binding des méthodes de traits** + protocoles bénis | 2 |
| 2.2 | 51700 | Robustesse du bind (ré-exports, collisions, getters, itérateurs) | 2 |
| 2.3 | 51800 | Inverse : export Iron → crate Rust (`ironc lib`) | 2 |
| 2.4 | 70700 | `import <crate>` (auto-bind + cache) | 2 |
| 2.5 | 70800 | Registre de bindings curés + `ironc add` | 2 |
| 3.1 | 40300 | Fix `try/except` + hoisting | 3 |
| 3.2 | 40400 | Auto-wrap I11 complet | 3 |
| 3.3 | 21300 | Inférence élargie (HOF fiable) | 3 |
| 3.4 | 10900 | Générateurs `yield` → itérateurs | 3 |
| 4.1 | 30700 | `shared_sync[T]` → `Arc<Mutex>` | 4 |
| 4.2 | 70900 | Module `thread` (spawn/join, closures move) | 4 |
| 4.3 | 21200 | `async`/`await` → Rust async + runtime | 4 |
| 5.1 | 60500 | Passe de lint réelle | 5 |
| 5.2 | 60600 | Erreurs de type niveau Iron | 5 |
| 5.3 | 60700 | Source-maps granularité expression | 5 |
| 5.4 | 71000 | `ironc fmt` | 5 |
| 5.5 | 71100 | LSP | 5 |
| 5.6 | 71200 | `ironc repl` | 5 |
| **6.1** | 90400 | Batterie « 30 vrais crates » | 6 |
| **6.2** | 90100 | Métrique de vérité chiffrée | 6 |
| **6.3** | 90500 | **Gate V9 final** | 6 |

## Fondation (vague 1, séquentielle, gel)

`1.1` gèle dans `ir.rs` : `Type::SharedSync(Box<Type>)` (30700), marqueur `is_async` sur `Item::Fn` +
`ExprKind::Await` (21200), `StmtKind::Yield` (10900), tag protocole sur les méthodes externes (51600).
Arms par défaut dans `infer`/`resolve`/`lower` → **build vert** sans logique. **Aucune vague 2+ ne re-touche
`ir.rs`** (trou de contrat → on remonte en 1.1).

## Parallélisme (test mental anti-collision)

- **Vague 2 (interop)** : surfaces `bind.rs`/`infer`/`lower` partagées → **séquentiel-incrémental** (corpus +
  gate binding verts entre pistes), sauf `2.3` (`ironc lib`, driver/CLI) et `2.5` (registre, dossier) →
  parallélisables (ownership disjoint).
- **Vague 3 (polish)** : `lower`/`infer` partagés → séquentiel.
- **Vague 4 (concurrence)** : `4.1`→`4.2`→`4.3` **ordonnées** (thread dépend de shared_sync ; async à part).
- **Vague 5 (diag ∥ tooling)** : **vrai fan-out** — `60xxx` (lint/typecheck/sourcemap) vs `71xxx`
  (fmt/lsp/repl) = sous-systèmes **disjoints**. `5.5` (LSP) dépend de `5.2` (erreurs Iron) + `5.3` (sourcemaps).
- **Vague 6 (validation)** : `6.1`→`6.2`→`6.3` séquentielles (la métrique agrège la batterie ; le gate agrège tout).

## Ordre d'exécution

1 (fondation) → 2 (interop, trait-binding d'abord) → 3 (polish cœur) → 4 (concurrence : shared_sync→thread→async)
→ 5 (lint/diag ∥ tooling) → 6 (batterie → métrique → gate V9).

> Invariant dur : corpus + gate binding **verts en permanence** ; chaque vague les re-valide avant la suivante.

## Gate V9 (= done de 6.3 = « langage fini à 99.9% »)

Projet-vitrine multi-fichiers (≥ 3 crates en `import`, threading `shared_sync`+`thread`, méthodes de traits
en Iron pur, `try/except` direct) compile+s'exécute, **0 `unsafe`** ; `fmt`/`lint`/`test`/`lib`/`lsp` OK ;
corpus + 30-vrais-crates verts ; **métrique de vérité chiffrée** publiée ; **tous les `🔶` tranchés**.
Reste seulement le **0.1% assumé** (HM complet, auto-hébergement, async exotique, binding garanti de tout
crate) — listé honnêtement dans `realisation.md`.
