# V8 — Planification (vagues & pistes)

> Découpage en **vagues séquentielles** (1→4) ; au sein d'une vague, **pistes** (`X.Y`). La fondation
> **D0/D1 gèle le format `.iri` v2 + le `Bindings` IR + la projection `map_type` + l'éclatement `bind/`** en
> vague 1 → les pistes suivantes remplissent les **producteurs** (fichiers `bind/*` disjoints) **sans
> re-toucher le contrat** (anti-collision, comme V6/V7). Gabarit : `generator §6`. Parente : `../README.md`.

## Table de correspondance piste → spec

| Piste | Spec | Titre | Vague |
|---|---|---|---|
| **1.1** | 50900 | **Fondation** : format `.iri` v2 + `Bindings` IR + éclatement `bind/` + pipeline `emit` | 1 |
| **1.2** | 51000 | **`map_type` récursif** (convention de projection partagée — figée avec la fondation) | 1 |
| **1.3** | 21100·40200 | **Squelette e2e** : infer + lower **minimal** (1 type externe + 1 méthode + idiome `clone`) | 1 |
| 2.1 | 51100 | Extraction POO : méthodes & constructeurs depuis `impl` | 2 |
| 2.2 | 51200 | Émission auto des types externes (fermeture du graphe) | 2 |
| 2.3 | 51300 | Reconnaisseurs d'idiomes (builder/getter/iterator/closure) | 2 |
| 2.4 | 51400 | Instance-pinning (monomorphisation à la demande) | 2 |
| 2.5 | 51500 | Namespacing (`extern mod` + chemins de module) | 2 |
| 3.1 | 21100 | Inférence **complète** des appels/méthodes/types externes | 3 |
| 3.2 | 40200 | Lowering **complet** des appels externes + glue des 4 idiomes | 3 |
| **4.1** | 90300 | Corpus de binding réel (regex/serde_json/csv) + **gate V8** | 4 |

## Fondation (vague 1, séquentielle, gel)

`1.1` gèle dans `ir.rs` **tout** le contrat neuf (`Bindings`/`ExternType`/`ExternFn`/`Recv`/`Idiom`/`ExternMod`,
cf. cadrage D0), écrit le **mini-parseur `.iri` v2** (`bind/mod.rs::parse_iri`), **éclate `bind.rs` en module
`bind/`** (`mod.rs`, `types.rs`, `methods.rs`, `emit_types.rs`, `idioms.rs`, `instantiate.rs`, `modules.rs` —
coquilles vides + arms par défaut), et garde le **build vert** + corpus passant. `1.2` fige `map_type`
(la convention de projection, D1). `1.3` câble la colonne e2e fine.

> Gelé = **aucune piste vague 2+ ne re-touche `ir.rs` ni `parse_iri` ni `map_type`**. Trou de contrat
> découvert en vague 2 → on remonte en `1.1`/`1.2`, on ne patche pas localement (règle V6/V7).

## Parallélisme (test mental anti-collision)

- **Vague 1 = séquentielle** (fondation : contrat + projection + squelette).
- **Vague 2 = vrai fan-out parallèle.** Chaque piste possède **un fichier `bind/*` disjoint** :
  `2.1`→`methods.rs`, `2.2`→`emit_types.rs`, `2.3`→`idioms.rs`, `2.4`→`instantiate.rs`, `2.5`→`modules.rs`.
  Toutes consomment `map_type` (1.2, figé) et mutent un `Bindings` via des fonctions **pures par fichier** que
  `emit` (dans `mod.rs`, possédé par la fondation) orchestre. **Aucune ne touche `mod.rs`/`types.rs`/`ir.rs`.**
- **Vague 3 = unification séquentielle.** `infer` (`3.1`) puis `lower` (`3.2`) sont des **surfaces partagées** →
  séquentielles-incrémentales (corpus vert entre les deux). `3.2` touche `lower`/`sourcemap` (I8) → isolé.
- **Vague 4 = gate** (corpus réel ; touche `tests/` + `gate.sh`).

## Ordre d'exécution

1 (fondation : `1.1`→`1.2`→`1.3`, séquentiel) → 2 (producteurs `bind/`, **parallèle**, ownership disjoint)
→ 3 (consommateurs : `3.1` infer puis `3.2` lower, séquentiel) → 4 (gate `4.1`).

> Invariant dur (méthodo phase 3) : la colonne e2e du squelette (`1.3`) **passe en permanence**. La vague 2
> produit des `.iri` plus riches sans casser la colonne ; la vague 3 les rend utilisables et **re-valide
> l'e2e réel** avant le gate.

## Gate V8 (= done de 4.1)

`ironc bind regex/serde_json/csv` → `.iri` riches ; ≥ 1 programme Iron par crate (POO style Python) **compile +
s'exécute + sortie = oracle** ; couverture mesurée **≥ 90 %** ; **0 `unsafe`** (I2) ; clones de getters comptés
`--audit` (I4) ; remontée I8 vérifiée sur un appel externe ; **aucune régression** V0→V7. Faiblesses →
`realisation.md` (et V9 : registre partagé, async, traits-protocoles).
