# [2.5] Lowering → TokenStream (40100)

- Spec parente : 40100 · V0 · **Vague 2** (parallèle) · Statut : todo
- Dépend de : 1.1 (types gelés + format de marqueur gelé)

## But
Remplir `src/lower.rs` : `Ast`+`TypeMap`+`OwnershipMap` → `(TokenStream, SourceMap)` via `proc_macro2` (I10, **jamais de chaînes**), tout le sous-ensemble V0 + marqueurs.

## Fichiers possédés
`src/lower.rs` (+ `tests/lower_*.rs`).

## Interface
Consomme `&Ast, &TypeMap, &OwnershipMap`. Produit `(TokenStream, SourceMap)` (types **gelés** `ir.rs`).

## Étapes
Lowering items/flow (`if/while/for/break/continue`)/exprs/collections (`Vec`/`HashMap`)/`Option`-`Result`-`?`/clone-move (`Clone`→`.clone()`) ; crate `#![forbid(unsafe_code)]` + `IronError` ; **marqueurs** par instruction (cible expression) liés au `NodeId`.

## Critère de done
Programme V0 → `TokenStream` que `cargo` compile (via 2.7) ; sémantique préservée (exécution = attendu) ; `SourceMap` alignée sur les `NodeId` ; tests verts ; build vert.

## Notes
Mécanisme de marqueur **prouvé en 1.1**, étendu ici à tout le langage. Ne touche **que** `lower.rs`.
