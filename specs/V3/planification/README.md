# Iron — V3 / planification (`README.md`)

> Découpe `20300` (types avancés). Stratégie : **projection + `rustc` filet** (cf. `V3/README.md §1`).
> Contrats gelés : `ir.rs` (nouveaux nœuds AST + `Type::Enum`/`Generic`). Banc : corpus V0/V1/V2.

| Piste | Périmètre | Modules |
|---|---|---|
| 1.1 | fondation : AST (Enum/Trait/Impl/Match/Pattern/génériques) + `Type` + stubs, corpus vert | `ir.rs`, passes |
| 2.1 | **enums + `match`** (lève la dette V0) | lex/parse/infer/lower |
| 2.2 | **traits + `impl`** + appel méthode | parse/infer/lower |
| 2.3 | **génériques** `[T]` (fn/struct/enum + bornes) | parse/lower |
| 3.1 | unification + gate V3 | — |

## Vagues
Séquentielles-incrémentales (les features partagent parser/infer/lower ; intégration continue) : 1.1 → 2.1 →
2.2 → 2.3 → 3.1. Chaque incrément garde le corpus **vert** et ajoute son gate.

## Gate V3 (= `V3/README.md §6`)
enum+match exécute · Option/Result par match · trait+impl appelé · générique ≥2 types · match non-exhaustif →
erreur remontée · zéro `unsafe` · corpus V0/V1/V2 vert.

## État de réalisation (2026-06-24)
**Gate V3 : ✅ vert, les 4 features** (cf. `../realisation.md`). `2.1` enums+match ✅ · `2.2` traits+impl ✅ ·
`2.3` génériques ✅. Preuves : `enum_match→12/12/42/-1`, `generics→42/true/7/10`, `traits→woof/woof/3`,
`match` non-exhaustif → `E0004` remonté. Corpus **18/18**. Dettes : `../realisation.md`.
