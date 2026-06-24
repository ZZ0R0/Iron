# Iron — V4 / planification (`README.md`)

> Découpe `80100` (stdlib = prélude de built-ins). Périmètre réduit → **1 piste**. Banc : corpus V0–V3.

| Piste | Périmètre | Modules |
|---|---|---|
| 2.1 | built-ins prélude (`str/abs/min/max/sqrt/push/sum/concat`) : inférence + lowering | `infer/local.rs`, `lower.rs` |
| 3.1 | gate : programme stdlib + corpus vert | — |

## Gate V4 (= `V4/README.md §5`)
Programme `str/abs/min/max/sqrt/push/sum/concat` → `cargo` vert + sortie correcte ; zéro `unsafe` ; corpus V0–V3 vert.
