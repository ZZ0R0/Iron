# Iron — V2 / réalisation (trace)

> Phase 3 terminée. **Gate V2 vert (2026-06-24).** Interop Rust, **les 3 niveaux** (I7).

## Gate V2 — atteint

| Niveau | Preuve |
|---|---|
| **1 — `rust { }` inline** (50100) | `rust_inline.irn` : `rust { let r = x.sqrt(); … }` → `3` (√9) |
| **2 — `.iri` + `use` + dépendance crate** (50200) | `use_math` → `4` (`f64::sqrt`, std) · `use_crate` → `5` (`libm::sqrt`, `libm="0.2"` dans le `Cargo.toml` généré) |
| **3 — `ironc bind` rustdoc-json** (50300) | `bind libmproj libm` → `.iri` (104 fns, **144 sautées/reportées**) → `use` → `sqrt(49.0)` → `7` |
| Zéro `unsafe` (I2) · corpus V0/V1 vert · tests | `forbid` ; corpus **11/11** ; `cargo test` 3/3 |

`#![forbid(unsafe_code)]` rend l'échappatoire `rust:` **incapable** d'introduire un `unsafe` (I2 gratuit).

## Pistes livrées

- **2.1 (50100)** : lexer capte `rust { … }` **brut** (`RustVerbatim`, brace-depth, saute chaînes/`//`) ;
  AST `RustBlock` ; lowering parse en `TokenStream` (I10) et insère.
- **3.1 (50200)** : module `iri.rs` (parseur de bindings) ; `use "x.iri"` ; externs fusionnés dans les
  signatures d'inférence (type-check) ; lowering appel→`rust::path` ; `Cargo.toml` `[dependencies]`.
- **4.1 (50300)** : `bind.rs` + `ironc bind <dir> [pkg]` : `cargo +nightly rustdoc` → JSON (`serde_json`) →
  projection des fns publiques mappables vers le sous-ensemble Iron, items non mappables **comptés**.
- **Bonus** : annotation de type des locales hoistées (corrige un défaut latent V1 : `{float}` ambigu sur appel de méthode).

## Écarts & dettes → cadrage V3

- **DW1 — `rust:` brace-only** (`rust { }`) ; forme indentée `rust:` repoussée. Capture brace-depth
  **naïve** (saute chaînes + `//`, pas `/* */` ni `'{'`). Pont de valeurs **par nom** (bas niveau, I7-1).
- **DW2 — type-check extern via `rustc`** : l'inférence Iron ne vérifie pas (encore) les types d'arguments
  d'appel contre la signature extern ; `rustc` est le filet (E0308 remonté, I2). À renforcer.
- **DW3 — `ironc bind` best-effort** : mapping **exact** `i64/f64/bool` seulement (reste sauté & reporté) ;
  chemin `crate::nom` (items en sous-module → chemin faux, mais re-exports OK) ; doublons possibles
  (libm émet `sqrt` 2×, inoffensif — `.iri` déduplique). Cible rustdoc-json **v57**.
- **DW4 — `Type::Extern`** câblé mais peu testé (le gate utilise des externs à retour primitif).
- **DW5 — bind d'une crate locale** émet une version non résolvable sur registre (attendu ; path-deps non
  supportés dans le format `.iri`).

## Phase 4 (boucle de vérité)
Corpus V0/V1 inchangé (non-régression) + 4 programmes interop e2e + reporting des items non mappables.
Métrique de vérité (corpus large) toujours différée.
