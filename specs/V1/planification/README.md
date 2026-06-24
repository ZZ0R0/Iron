# Iron — V1 / planification (`README.md`)

> Découpe la spec `30200` en unités `planify-X.Y` (X = vague, Y = piste).
> Contrats gelés : `V1/README.md §2–3` + `ir.rs` (extension `Disposition::Borrow`, `ParamModes`).
> **Banc de non-régression sémantique** : le corpus `ironc/tests/fixtures/` (gate critère 1).

## Correspondance piste → module

| Piste | Spec | Module **possédé** |
|---|---|---|
| 1.1 | 30200 (fondation) | `ir.rs` (+`Borrow`, +`ParamModes`) · stubs `resolve`/`lower` MAJ · flag `--audit` |
| 2.1 | 30200 | `resolve.rs` : escape analysis (`ParamModes` + `OwnershipMap`) |
| 2.2 | 30200 | `lower.rs` : respecte `ParamModes`/`Disposition` + émet `--audit` |
| 3.1 | unification | corpus identique + moins de clones |

## Vagues

- **1.1 (séquentiel)** : fige le contrat (`ir.rs`). `resolve` renvoie **all-Clone** (= comportement V0) →
  **le corpus reste vert sans rien changer**. `lower` apprend à gérer `Borrow`/`Move`/`ByRef` (chemins prêts).
- **2.1 + 2.2 (parallèle)** : escape analysis réelle (2.1) ; `lower` réel signatures+call-sites (2.2) ;
  contre contrat gelé + fixtures.
- **3.1 (séquentiel)** : unification → gate V1.

## Gate V1 (= `V1/README.md §5`)
Corpus **11/11 sorties identiques** à V0 · **strictement moins de `.clone()`** · `--audit` liste les
fallbacks · zéro `unsafe` · compile toujours (I1).

## État de réalisation (2026-06-24)
**Gate V1 : ✅ vert** (cf. `../realisation.md`). `1.1`/`3.1` ✅ · `2.1` escape analysis ✅ · `2.2` lowering
emprunts + `--audit` ✅. Mesure : corpus **11/11 identiques**, clones **4 → 0**. Dettes : `../realisation.md`.
