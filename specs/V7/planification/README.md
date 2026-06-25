# V7 — Planification (vagues & pistes)

> Découpage en **vagues séquentielles** (1→6) ; au sein d'une vague, **pistes** (`X.Y`). La fondation
> **D0 gèle tout l'IR neuf en vague 1** → les vagues suivantes remplissent la logique **sans re-toucher
> `ir.rs`** (anti-collision, comme V6). Gabarit : `generator §6`. Parente : `../README.md`.

## Table de correspondance piste → spec

| Piste | Spec | Titre | Vague |
|---|---|---|---|
| **1.1** | — | **Fondation D0** : gel de tout l'IR neuf | 1 |
| 2.1 | 10400 | Tuples + destructuring + retour multiple | 2 |
| 2.2 | 10500 | f-strings | 2 |
| 2.3 | 20800 | Zoo numérique + `char` | 2 |
| 2.4 | 10700 | Affectation augmentée + opérateurs | 2 |
| 2.5 | 10800 | Ternaire / `in` / `assert` / slicing | 2 |
| 2.6 | 20900 | Type `set` | 2 |
| 2.7 | 10600 | Compréhensions (dép. 2.1, 2.6) | 2 |
| 2.8 | 21000 | Variantes à champs nommés | 2 |
| 3.1 | 70300 | Manifeste `ironc.toml` | 3 |
| 3.2 | 50500 | `bind` complet | 3 |
| 3.3 | 50600 | Dépendances de crates + auto-bind (dép. 3.1, 3.2) | 3 |
| 3.4 | 50800 | `rust:` indenté | 3 |
| 3.5 | 50700 | Fns génériques + traits std | 3 |
| 4.1 | 80300 | Stdlib `io` (std) | 4 |
| 4.2 | 80500 | Collections (std) | 4 |
| 4.3 | 80600 | Combinateurs `Option`/`Result` | 4 |
| 4.4 | 80700 | Itertools (dép. 2.1 tuples) | 4 |
| 4.5 | 80400 | JSON (dép. 3.3 deps) | 4 |
| 4.6 | 80800 | datetime/random/regex/os (dép. 3.3) | 4 |
| 5.1 | 60300 | Passe de lint réelle | 5 |
| 5.2 | 60400 | Erreurs de type Iron | 5 |
| 5.3 | 60200 | Source-maps expression | 5 |
| 5.4 | 70400 | `ironc new` | 5 |
| 5.5 | 70500 | `ironc test` | 5 |
| 5.6 | 70600 | `ironc fmt` | 5 |
| 6.1 | 30500 | Escape point-fixe + for-var | 6 |
| 6.2 | 30600 | `weak()`/`Weak` + lint aliasing (dép. 5.1) | 6 |
| 6.3 | 90200 | Durcissement corpus + **gate V7** | 6 |

## Fondation D0 (vague 1, séquentielle, gel)

`1.1` gèle dans `ir.rs` **tous** les contrats neufs, puis câble des **arms par défaut** dans les `match`
exhaustifs (`infer`/`resolve`/`lower`) pour rester **build vert** sans logique :
- **Types** : `Type::Tuple(Vec<Type>)`, `Type::Set(Box<Type>)`, `Type::Num(NumKind)` (zoo hors i64/f64),
  `Type::Char`, `Type::Weak(Box<Type>)`. (`deque`/ordered → `Type::App`, pas d'ajout.)
- **Exprs** : `ExprKind::{Tuple(Vec<Expr>), FString(Vec<FStringPart>), Comp{…}, Ternary{…}, Slice{…}}`.
- **Motifs/stmts** : `Pattern::Tuple(Vec<Pattern>)`, `StmtKind::Assert{cond,msg}`, **cible destructurée**
  (`StmtKind::Destructure{targets, value}`).
- **Enum** : variantes à **champs nommés** (repr `Vec<(Option<String>, Type)>` ou variante dédiée).
- **BinOp** : `Pow, FloorDiv, BitAnd, BitOr, BitXor, Shl, Shr, In, NotIn`.
- **NumKind** : `I8…I128, Isize, U8…U128, Usize, F32` (i64/f64 restent `Type::I64/F64`).

> Gelé = **aucune vague 2+ ne re-touche `ir.rs`**. Si une piste découvre un trou de contrat → on remonte
> en 1.1, on ne patche pas localement (règle V6).

## Parallélisme (test mental anti-collision)

- **Vagues 2–4 = surfaces partagées** (`parse.rs`/`infer`/`lower`) → **séquentielles-incrémentales**
  (corpus vert entre chaque piste), comme V6. Exceptions parallélisables (ownership disjoint) : `50800`
  (lexer `rust:`), `70300/70400` (manifeste/CLI), les `.iri` curés de stdlib.
- **Vague 5 = D (lint/diag) ∥ E (tooling)** : sous-systèmes **disjoints** (`lint.rs`/`typecheck.rs` vs
  `main.rs`/fmt) → vrai fan-out parallèle. `5.3` (source-maps expr) touche `lower`/`sourcemap` → à isoler.
- **Vague 6** : finition ownership (`resolve.rs`) puis gate.

## Ordre d'exécution

1 (fondation) → 2 (langage, séquentiel-incrémental, 2.7 après 2.1+2.6) → 3 (crates ; 3.3 après 3.1+3.2)
→ 4 (stdlib ; 4.5/4.6 après 3.3) → 5 (lint/diag ∥ tooling) → 6 (finition + gate).

## Gate V7 (= done de 6.3)

Projet-démo réel (`ironc.toml` + crate + stdlib + tuples/f-strings/compréhensions) compile + s'exécute ;
`ironc new/test/fmt` OK ; lint juste ; **0 `unsafe`** ; clones maîtrisés ; corpus durci vert ; remontée I8
préservée. Faiblesses → `realisation.md` (et V8).
