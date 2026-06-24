# [2.3] Inférence : locale + Option/Result (20100, 20200)

- Spec(s) parente(s) : 20100, 20200 · V0 · **Vague 2** (parallèle) · Statut : todo
- Dépend de : 1.1 (types gelés + fixtures d'AST)

## But
Remplir `src/infer/` : `Ast` → `TypeMap` — monotypes/structs/collections (20100) puis `Option`/`Result` par flux (20200).

## Fichiers possédés
`src/infer/` (`local.rs`, `flow.rs`, `mod.rs`) (+ `tests/infer_*.rs`).

## Interface
Consomme `&Ast`. Produit/étend `TypeMap : NodeId→Type` (type **gelé** `ir.rs`).

## Étapes
**Local** : inférence ascendante (littéraux, vars, ops, struct, `list`/`dict`, `e[i]`, appels) ; vérif des annotations → erreur au `Span` si mismatch (I2). **Flux** : `fn` à chemin sans valeur → `Option<T>` ; `?`/op faillible → `Result<T,IronError>` (I13) ; `NaN`→`F64` (I12).

## Critère de done
Tout `Expr` du programme V0 typé ; mismatch annotation → erreur au `Span` ; `fn` sans-valeur→`Option`, `fn` avec `?`→`Result` ; `NaN`→`F64` ; tests verts ; build vert.

## Notes
Collection vide → annotation requise (cf. risques 20100). Consommation terminale Option/Result = limite V0 (cf. risques 20200). Ne touche **que** `src/infer/`.
