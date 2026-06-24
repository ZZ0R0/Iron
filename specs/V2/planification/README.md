# Iron — V2 / planification (`README.md`)

> Découpe `50100`/`50200`/`50300` (interop, I7). **I7 impose l'ordre** des niveaux → vagues **séquentielles**
> (peu de parallélisme ici, c'est voulu). Contrats gelés : `V2/README.md §4` + `ir.rs` (`RustBlock`, `Extern`).
> Banc de non-régression : corpus V0/V1 (`ironc/tests/fixtures/`).

| Piste | Spec | Module |
|---|---|---|
| 1.1 | fondation | `ir.rs` (+`StmtKind::RustBlock`, +`Type::Extern`), `Cargo.toml` `[dependencies]` plumbing, corpus vert |
| 2.1 | 50100 | lexer (`rust { }` brut) + parser + lowering verbatim |
| 3.1 | 50200 | parser `.iri` + `use` + type-check extern + dépendance crate |
| 4.1 | 50300 | `ironc bind` : rustdoc-json → `.iri` |

## Vagues (séquentielles, I7)
1.1 fondation → 2.1 niveau 1 → 3.1 niveau 2 → 4.1 niveau 3. Chaque niveau garde le corpus vert et ajoute son
gate (`V2/README.md §6`). On ne commence un niveau qu'après le précédent vert.

## Gate V2 (= `V2/README.md §6`)
`rust {}` appelant du Rust réel ✅ · `.iri` importé+type-checké+dépendance crate ✅ · `ironc bind` génère un
`.iri` valide ✅ · zéro `unsafe` (forbid) · corpus V0/V1 vert.

## État de réalisation (2026-06-24)
**Gate V2 : ✅ vert, les 3 niveaux** (cf. `../realisation.md`). `2.1` `rust:` inline ✅ · `3.1` `.iri`+`use`+crate ✅ ·
`4.1` `ironc bind` rustdoc-json ✅. Preuves : `rust_inline→3`, `use_math→4`, `use_crate→5` (libm), `use_gen→7`
(bind→use). Dettes : `../realisation.md`.
