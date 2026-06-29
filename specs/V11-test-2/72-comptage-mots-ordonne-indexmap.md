# 72 — Comptage de mots à ordre préservé
> **Domaine** Données · **Crate(s)** `ironc add indexmap` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Compter les occurrences de mots d'un texte en **préservant l'ordre d'insertion** (`IndexMap<String, i64>`),
afficher le top par ordre d'apparition.

## Ce que ça teste dans Iron
`IndexMap<K, V>` est un **générique à 2 paramètres** (comme `HashMap`, mais type de crate). Teste si le
binder projette une map externe générique et son API (`insert`, `entry`/`or_insert`, itération ordonnée),
là où `dict` Iron est la std `HashMap`.

## Approche
`IndexMap.new()`, `insert`/`get`, itération `keys()`/`values()`. Mapper sur des boucles Iron. Cibler les
méthodes concrètes (pas l'API `entry` à closure si non projetée).

## Done
`ironc run` compte les mots d'une phrase et imprime les comptes dans l'ordre d'apparition.
