# Iron — V6 / cadrage (`README.md`)

> **Version de FINITION** : les **18 manques** de l'audit V0–V5, écrits proprement en specs. Aucune
> nouvelle ambition de roadmap — on **clôt les invariants partiels et les dettes**. Parent :
> `specs/generated.md` (V5/V6 ajoutées). Contrat : `../generator.md`.
> Statut : **🔶 cadrage** (specs écrites ; planification + réalisation à suivre sur feu vert).

## Carte des specs (18 items → 12 specs)

| Spec | Items de l'audit | Invariant / dette |
|---|---|---|
| `20400` Inférence Option/Result par le flux + `try/except` | 1 | **I11** |
| `30300` Partage explicite `ref`/`Rc`/`Weak` + warning aliasing | 2, 4 | **I5**, **I3** |
| `20700` Closures / lambdas + `map`/`filter` | 12 | langage |
| `20500` Inférence des types de paramètres | 5 | 20100 |
| `20600` Motifs `match` avancés (gardes, champs nommés, imbrication) | 7a | 20300 |
| `30400` Ownership avancé (escape point-fixe, move-hors-struct, scoping `for`) | 6, 7b, 17 | 30200, **I1** |
| `60200` Source maps granularité **expression** | 3 | **I8** |
| `50400` Interop élargi (`bind`, `rust:` indenté, `Extern` opaques) | 8, 10, 11 | 50x |
| `70200` Modules multi-fichiers Iron (`import`) | 14 | langage |
| `80200` Stdlib élargie (`sum`/`concat` génériques, tri, IO, chaînes) | 9, 13, 15 | 80100 |
| `10300` Parser : comparaison chaînée `a < b < c` | 16 | 10200 |
| `90100` Corpus de validation (métrique de vérité large) | 18 | méthodo |

## Priorité (structurant → petit)
1. **Structurants** : `20400` (I11 flux), `30300` (I5 Rc), `20700` (closures).
2. **Inférence / ownership / diagnostics** : `20500`, `20600`, `30400`, `60200`.
3. **Interop / modules / stdlib / parser** : `50400`, `70200`, `80200`, `10300`.
4. **Méta** : `90100` (différé jusqu'au gel du langage).

## Note
Specs au gabarit `generator §4`. La planification (vagues) viendra ensuite ; ordre conseillé : invariants
d'abord, confort ensuite.
