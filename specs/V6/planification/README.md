# Iron — V6 / planification (`README.md`)

> Phase 2. Découpe les **12 specs** V6 en unités `planify-X.Y`. **Particularité** : les features touchent des
> passes **partagées** (`infer`/`lower`/`parse`/`resolve`) → réalisation surtout **séquentielle-incrémentale**
> (corpus toujours vert), pas un fan-out parallèle. **Fondation d'abord** (gel des nouveaux contrats IR).
> Banc : corpus **22 programmes** (V0–V5). Gabarit : `generator §6`.

## Correspondance piste → spec

| Piste | Spec | Item(s) | Module(s) possédé(s) | Dépend |
|---|---|---|---|---|
| 1.1 | fondation | — | `ir.rs` (+ arms défaut) | — |
| 2.1 | 20400 (I11 flux) | 1 | `infer/flow.rs`, `lower.rs`, `parse.rs` | 1.1 |
| 2.2 | 30300 (Rc/ref) | 2,4 | `ir.rs`, `lex/parse`, `lower.rs` | 1.1 |
| 2.3 | 20700 (closures) | 12 | `ir`, `parse`, `infer`, `lower` | 1.1 |
| 3.1 | 20500 (inf. params) | 5 | `infer/` | 1.1 |
| 3.2 | 20600 (motifs match) | 7a | `ir`, `parse`, `lower` | 1.1 |
| 3.3 | 30400 (ownership avancé) | 6,7b,17 | `resolve.rs`, `lower.rs` | 1.1 |
| 3.4 | 60200 (source maps expr) | 3 | `lower.rs`, `sourcemap.rs` | 1.1 |
| 4.1 | 50400 (interop élargi) | 8,10,11 | `bind.rs`, `lex.rs`, `iri.rs` | 1.1 |
| 4.2 | 70200 (modules) | 14 | `driver.rs`, `parse.rs` | 1.1 |
| 4.3 | 80200 (stdlib élargie) | 9,13,15 | `infer/local.rs`, `lower.rs` | 1.1 |
| 4.4 | 10300 (comparaison chaînée) | 16 | `parse.rs` | 1.1 |
| 5.1 | 90100 (corpus validation) | 18 | `corpus/`, runner | toutes |
| 6.1 | unification + gate V6 | — | — | 2.x–4.x |

## Vagues (ordre)
1. **1.1 fondation** : gel des nouveaux contrats IR.
2. **2.x structurants** (séquentiels, risqués, partagés) : `20400` → `30300` → `20700`.
3. **3.x inférence / ownership / diagnostics** : `20500`, `20600`, `30400`, `60200`.
4. **4.x interop / modules / stdlib / parser** : `50400`, `70200`, `80200`, `10300` (4.4 et `bind` de 4.1
   **isolés** → parallélisables ; le reste séquentiel).
5. **5.1 méta** : `90100` (différé — gel du langage requis).
6. **6.1 unification** + gate V6.

## Anti-collision
Ownership **surtout séquentiel** (`lower`/`infer`/`parse` partagés entre pistes) : chaque piste laisse le
corpus **vert** avant de passer à la suivante. Pistes **isolées** (`bind.rs`, `parse.cmp_expr`) parallélisables.

## Gate V6 (= `V6/README.md`)
Chaque spec a sa preuve · corpus 22 inchangé · zéro `unsafe` · `cargo test`. Trace → `V6/realisation.md`.
