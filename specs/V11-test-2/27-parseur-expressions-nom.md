# 27 — Parseur d'expressions arithmétiques (combinateurs)
> **Domaine** Parsing · **Crate(s)** `ironc add nom` · **Risque Iron** 🟥 · **Difficulté** élevée

## But
Parser une expression arithmétique simple (`1 + 2 * 3`) en arbre/valeur via des **combinateurs de parseurs**
et l'évaluer.

## Ce que ça teste dans Iron
`nom` est **closure/trait-lourd** : un parseur est une `Fn(Input) -> IResult<Input, Output>`. Teste la
composition de fonctions d'ordre supérieur, les types d'erreur génériques et les combinateurs (`tag`, `alt`,
`map`, `delimited`). Cas dur attendu pour le binder feuille.

## Approche
`.iri` curé probable exposant quelques combinateurs concrets sur `&str`. À défaut, mesurer la **couverture**
de `ironc add nom` (ce que le binder sait extraire) et le rapport des sautés.

## Done
Au minimum : `ironc add nom` produit un binding + rapport ; idéalement un mini-parseur évalue `1+2*3 = 7`.
