# 67 — Distances géographiques & itinéraires (haversine)
> **Domaine** Géo, temps & docs · **Crate(s)** `ironc add geo` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Calculer la distance entre coordonnées (formule de **haversine**, et/ou géodésique), puis la **longueur d'un itinéraire** (somme des segments d'une polyligne). Bonus : ordonner des points par proximité.

## Ce que ça teste dans Iron
- Le motif **méthode-de-trait sur géométrie générique** : `HaversineDistance`/`GeodesicDistance` sont des **traits d'extension** appliqués à `Point<f64>` / `LineString`.
- L'import de **prélude** (`geo::prelude::*`) et les **impls blanket** génériques (`CoordFloat`).
- Maths flottantes, construction de `Point`/`LineString`, itération sur un `Vec` de coordonnées.

## Approche
Construire des `Point` depuis (lat, lon), appeler la distance via le trait, assembler une `LineString` pour l'itinéraire et sommer les segments. Comparer deux villes connues à une valeur de référence (à l'arrondi près).

## Risque / limites attendues
- 🟡 La friction est l'**API par traits d'extension** : le binder auto **surface mal** une méthode ajoutée par un trait + impl blanket → un `.iri` exposant `haversine_distance(a, b)` comme appel direct sera sans doute nécessaire.
- `Point<T>` **générique** : Iron doit fixer `T = f64`.
- Plusieurs métriques (haversine vs géodésique Vincenty) aux résultats légèrement différents — choisir et documenter laquelle.

## Done
- Distance d'un couple de villes connu conforme à la référence (à l'arrondi).
- Longueur d'itinéraire = somme correcte des segments.
- Zéro `unsafe` ; choix de la métrique explicité.
