# 06 — Convertisseur CSV → JSON
> **Domaine** Données & sérialisation · **Crate(s)** `ironc add csv` `ironc add serde_json` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Lit un fichier CSV avec en-tête et le convertit en tableau JSON d'objets (une clé par colonne). Entrée = `.csv` ; sortie = `.json` indenté, chaque ligne devenant `{ "col": "val", ... }`.

## Ce que ça teste dans Iron
Deux piliers de l'écosystème. On teste csv **sans la voie serde-derive** (qui exigerait une struct dérivée) : lecture par `StringRecord`. Et la **construction dynamique** d'un `serde_json::Value` (enum + map) plutôt que la sérialisation d'un type natif — un bon test des enums Iron et de l'interop map.

## Approche
- csv : `Reader::from_path(path)`, récupérer `.headers()` puis itérer `.records()` → `StringRecord`.
- Zipper en-têtes et champs dans une `serde_json::Map`, l'envelopper en `Value::Object`, pousser dans un `Vec<Value>`.
- `serde_json::to_string_pretty(&values)` → écrire le fichier.

## Risque / limites attendues
Faible **si** on évite `reader.deserialize::<T>()` (qui réclame `#[derive(Deserialize)]`, hors de portée du binder). En restant sur `StringRecord` + construction manuelle de `Value`, tout est dynamique et passe. À valider : l'enum `serde_json::Value` côté Iron et l'insertion dans `Map` (emprunt mutable). Limite assumée : typage CSV → tout en string (pas d'inférence nombre/bool). 🟢.

## Done
Un CSV `name,age` / `Alice,30` produit `[{"name":"Alice","age":"30"}]`. Cas de test : 2 lignes de données donnent un tableau de 2 objets aux bonnes clés, et un CSV sans données donne `[]`.
