# [2.2] Parser + AST (10200)

- Spec parente : 10200 · V0 · **Vague 2** (parallèle) · Statut : todo
- Dépend de : 1.1 (types gelés + fixtures de tokens)

## But
Remplir `src/parse.rs` : `Vec<Token>` → `Ast`, attribution `NodeId` (pré-ordre DFS), **2 syntaxes de bloc → même AST**, désambiguïsations (10200).

## Fichiers possédés
`src/parse.rs` (+ `tests/parse_*.rs`).

## Interface
Consomme `Vec<Token>`. Produit `Ast` (type **gelé** `ir.rs`).

## Étapes
Grammaire V0 ; bloc `:`-indenté **ou** `{}` → même `body` ; `{…}` bloc vs `{k:v}` dict **par position** ; `S(x=…)`→`StructLit`, `f(…)`→`Call` ; `NodeId` DFS dense ; `Span` nœud = 1ᵉʳ→dernier token ; erreurs au `Span`.

## Critère de done
Les **2 écritures (`:` et `{}`) → AST identique** (hors spans) ; `NodeId` denses/pré-ordre ; `{1:2}` (expr)→`DictLit`, `{stmt}` (bloc)→bloc ; `S(x=1)`→`StructLit`, `f(1)`→`Call` ; tests verts ; build vert.

## Notes
Imbrication inter-styles : règle à figer avec 2.1 (cf. risques 10200). Ne touche **que** `parse.rs`.
