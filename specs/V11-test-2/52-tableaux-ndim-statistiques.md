# 52 — Tableaux N-dimensionnels & statistiques
> **Domaine** Maths/vectoriel · **Crate(s)** `ironc add ndarray` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Créer des tableaux 1D/2D, faire de l'arithmétique élément-par-élément et du broadcasting (`a + b`,
`a * 2.0`), des réductions (somme, moyenne, écart-type, min/max par axe) et quelques slices.

## Ce que ça teste dans Iron
`ndarray` est générique sur l'**élément** et la **dimension** (`Array<A, D>`, avec `Ix1`/`Ix2`/`IxDyn`).
Deux pièges propres au binder : (1) le slicing idiomatique passe par la **macro `s![ ]`** — or Iron ne
sait pas appeler une macro Rust, c'est un **angle mort** ; (2) les stats vivent dans un crate annexe
(`ndarray-stats`) via des **traits d'extension** (`SummaryStatisticsExt`) dont les méthodes n'apparaissent
pas sans import explicite. Bon terrain, par ailleurs, pour la surcharge d'opérateurs (V11).

## Approche
`.iri` ciblé sur les alias monomorphes `Array1<f64>` / `Array2<f64>` : constructeurs (`from_vec`,
`zeros`, `from_shape_vec`), opérateurs `+ - * /` (V11) pour l'élément-par-élément, réductions inhérentes
(`sum`, `mean`, `std`). Remplacer la macro `s![ ]` par des accès **méthode** (`row`, `column`,
`index_axis`, `slice` via objets `Slice`). Brancher `ndarray-stats` en liant explicitement les méthodes
de trait voulues. Cas e2e : moyenne/écart-type par colonne d'un `Array2`.

## Risque / limites attendues
- **Macro `s![ ]` absente** : tout slicing avancé doit être ré-exprimé en méthodes → ergonomie réduite.
- La **dimension générique `D`** force à figer 1D/2D dans le `.iri` ; `IxDyn` (rang dynamique) reste fragile.
- Stats par **traits d'extension** : le binder ne les voit pas seul, il faut les déclarer une à une.

## Done
`ironc run` charge un `Array2`, calcule moyenne/écart-type par colonne, fait `a + b` et `a * 2.0`, imprime
un résumé cohérent. Compile sans `unsafe`.
