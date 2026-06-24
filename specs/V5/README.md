# Iron — V5 / cadrage (`README.md`)

> **Version de complétude** : résorber les manques de l'audit (invariants partiels + dettes). Pas une
> nouvelle feature de roadmap — on **honore I1–I14** et on **paie les dettes** `V*/realisation.md`.
> Parent : `specs/generated.md` (V5 ajoutée). Contrat : `../generator.md`. Statut : **🔶 en cours**.

## Cibles (par valeur / risque)

| # | Cible | Invariant / dette | Risque |
|---|---|---|---|
| G1 | **`move` à la dernière utilisation** (liveness) | **I1** (DV1) | moyen |
| G2 | **Inférence d'appel de méthode** (type de retour réel) | DX3 | faible |
| G3 | **`inout`/`ref` + `&mut` + warning partage** | **I3** (non fait) | moyen |
| G4 | **Inférence Option/Result par le flux** | **I11** (annotation→flux) | élevé |
| G5 | **Source maps granularité expression** | **I8** (instruction→expr) | moyen |
| G6 | **Test `NaN ≠ null`** | **I12** (non testé) | trivial |
| G7 | **Partage explicite `ref`** (`Rc`) | **I5** (non fait) | élevé |
| G8 | **Métrique de vérité** : corpus + comptage clones/correction | méthodo (différée) | mesure |

## Principe
Incréments **toujours-verts** : chaque cible garde le corpus (19 prog.) vert + ajoute sa preuve. Ce qui
résiste proprement est **fait à l'arrache + noté** ; ce qui est trop gros pour être sûr est **scopé honnêtement**.

## Gate V5
Corpus inchangé · les preuves par cible · zéro `unsafe` · `cargo test`. Trace dans `realisation.md`.
