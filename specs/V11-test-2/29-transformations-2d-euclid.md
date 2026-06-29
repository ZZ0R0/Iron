# 29 — Transformations géométriques 2D typées
> **Domaine** Géométrie · **Crate(s)** `ironc add euclid` · **Risque Iron** 🟡 · **Difficulté** élevée

## But
Créer des points/vecteurs 2D, appliquer translation/rotation/échelle, composer des transformations et
transformer un point.

## Ce que ça teste dans Iron
`euclid` est **doublement générique + phantom-typé** : `Point2D<T, Unit>` porte un paramètre d'**unité
fantôme** (espace de coordonnées) en plus du scalaire. Teste la projection d'un générique « marqueur » sans
valeur runtime — proche du cas C1 (alias monomorphisés) mais avec un type-marqueur.

## Approche
Cibler les alias `default::Point2D<f64>` / `default::Transform2D<f64>` (unité par défaut, monomorphe).
`Point2D.new(x, y)`, `Transform2D.translation(dx, dy)`, `.transform_point(p)`.

## Done
`ironc run` transforme un point par une composition et imprime les coordonnées attendues.
