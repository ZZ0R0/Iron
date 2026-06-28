# 13 — Mini moteur de requêtes sur DataFrame (filter/groupby)
> **Domaine** Bases de données · **Crate(s)** `ironc add polars` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Charger un CSV en DataFrame puis exécuter une requête analytique : filtrer des lignes, grouper par une colonne, agréger (somme/moyenne/compte). En entrée : un CSV + une requête simple ; en sortie : un DataFrame résultat affiché en table. Le scope reste minimal (une requête), mais le crate est un monstre.

## Ce que ça teste dans Iron
Le pire cas pour le binder : une **API lazy à DSL d'expressions** (`col("x").gt(lit(5))`), bâtie sur la **surcharge d'opérateurs d'`Expr`**, des **builders chaînés** (`.filter().group_by().agg().collect()`) et une avalanche de **bornes génériques** (`IntoLazy`, `IntoExpr`). C'est la frontière « génériques/builders/lazy » annoncée.

## Approche
- Lecture : `CsvReadOptions::default().try_into_reader_with_file_path(Some(path))?.finish()` → `DataFrame` (builder).
- Requête : `df.lazy().filter(col("age").gt(lit(18))).group_by([col("city")]).agg([col("age").mean()]).collect()`.
- `col()` / `lit()` construisent des `Expr` combinés par opérateurs (`>`, `+`) et méthodes (`.sum()`, `.alias()`).

## Risque / limites attendues
- La **surcharge d'opérateurs sur `Expr`** suppose qu'Iron mappe `>`/`+` vers les traits de polars : peu probable via binder auto → DSL à reconstruire à la main en `.iri`/wrapper, au prix de l'ergonomie.
- `LazyFrame` + `collect()` paresseux, les **namespaces** (`.str()`, `.dt()`) et les types `Series`/`ChunkedArray` génériques sont quasi inbindables tels quels.
- Réaliste : n'exposer qu'un **sous-ensemble** (lecture CSV + une poignée d'agrégats nommés) via un wrapper Iron étroit ; le DSL complet est hors d'atteinte.

## Done
Même minimal : charger un CSV de N lignes, produire un group-by + agrégation correcte, l'afficher. Cas de test : 6 lignes sur 2 villes, `group_by(city).agg(count)` rend 2 lignes aux bons comptes. **Documenter ce qui n'a pas pu être bindé.**
