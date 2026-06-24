# [2.4] Résolveur clone/move (30100)

- Spec parente : 30100 · V0 · **Vague 2** (parallèle) · Statut : todo
- Dépend de : 1.1 (types gelés + fixtures Ast/TypeMap)

## But
Remplir `src/resolve.rs` : `Ast`+`TypeMap` → `OwnershipMap` — clone par défaut (I1) + move à la dernière utilisation.

## Fichiers possédés
`src/resolve.rs` (+ `tests/resolve_*.rs`).

## Interface
Consomme `&Ast, &TypeMap`. Produit `OwnershipMap : NodeId→Disposition` (type **gelé** `ir.rs`).

## Étapes
Analyse de durée de vie locale par `fn` ; **dernière utilisation** sur **tous** les chemins → `Move`, sinon `Clone` ; usage **dans** une boucle → `Clone` (conservateur) ; `for x in coll` → clone élément.

## Critère de done
Tout `Var` lu a une `Disposition` ; binding 1× fin de portée → `Move`, 2× → `Clone` puis `Move` ; programme entièrement clonable compile (fallback I1) ; tests verts ; build vert.

## Notes
« Dernière utilisation » à travers `if`/`while`/`for` : règle conservatrice (cf. risques 30100). `Borrow` = hors V0. Ne touche **que** `resolve.rs`.
