# 73 — Index d'ensembles compressés (Roaring)
> **Domaine** Structures de données · **Crate(s)** `ironc add roaring` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Construire deux ensembles d'entiers compressés (`RoaringBitmap`), calculer union/intersection, cardinalité.

## Ce que ça teste dans Iron
`RoaringBitmap` est non-générique mais à **API ensembliste riche** (opérateurs `&`/`|`, `insert`, `len`,
itération) sur des `u32`. Teste le mappage d'opérateurs ensemblistes et l'itération d'octets/entiers.

## Approche
`RoaringBitmap.new()`, `insert(n)`, `&`/`|` (ou `intersection_len`/`union_len`), `len()`. Cibler les
méthodes concrètes.

## Done
`ironc run` imprime la cardinalité de l'intersection de deux ensembles (valeur connue).
