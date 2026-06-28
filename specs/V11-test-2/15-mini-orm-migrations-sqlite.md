# 15 — Mini-ORM + migrations sur SQLite
> **Domaine** Bases de données · **Crate(s)** `ironc add rusqlite` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Une fine couche ORM au-dessus de rusqlite : un **runner de migrations** versionné (suite de scripts SQL appliqués dans l'ordre, idempotents) et un **repository typé** par entité (`save/find/all/delete` mappant struct ↔ table). En entrée : des objets Iron ; en sortie : des lignes persistées et relues comme objets.

## Ce que ça teste dans Iron
Pas le binding (rusqlite est plat) mais **les abstractions d'Iron elles-mêmes** : un trait `Model` (`table_name`, `from_row`, `to_params`) et un `Repository<T>` générique. Stresse **génériques + traits** d'Iron, et l'**absence de `derive`** (le mapping ligne↔struct est écrit à la main).

## Approche
- Migrations : table `schema_version` (ou pragma `user_version`) ; lire la version courante, appliquer en boucle les migrations d'index supérieur, chacune en `execute`.
- Trait `Model` : `from_row(Row) -> Self` + `to_params() -> [valeurs]` + `table_name`.
- `Repository<T: Model>` : `save`, `find(id)`, `all()`, `delete(id)`, composant SQL + `query_map` sur le mapping du trait.

## Risque / limites attendues
- Sans `derive`, **chaque entité écrit son mapping à la main** : verbeux, et c'est précisément ce qu'on veut éprouver (jusqu'où vont traits/génériques d'Iron).
- Un `Repository<T>` réellement générique suppose des **bornes de trait** sur `T` et un `Vec<T>` construit via la closure de `query_map` — point de friction probable.
- Pas de réflexion ni de macro : la génération SQL reste **manuelle/string** côté Iron (le codegen Rust, lui, reste propre).

## Done
3 migrations appliquées **en séquence et idempotentes** (rejouer = no-op) ; un `Repository<User>` fait `save → find → all → delete`. Cas de test : base vide → migrer à v3 → insérer 2 users → `all()` = 2 → `delete` l'un → `all()` = 1 ; relancer les migrations ne change rien.
