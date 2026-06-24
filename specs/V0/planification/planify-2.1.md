# [2.1] Lexer (10100)

- Spec parente : 10100 · V0 · **Vague 2** (parallèle) · Statut : todo
- Dépend de : 1.1 (types gelés + fixtures)

## But
Remplir `src/lex.rs` : `.irn` → `Vec<Token>` réel — off-side rule + suspension dans les accolades, `Span` octets — conforme à 10100.

## Fichiers possédés
`src/lex.rs` (+ `tests/lex_*.rs`).

## Interface
Consomme `&str, FileId`. Produit `Vec<Token>` (type **gelé** `ir.rs`).

## Étapes
Scanner littéraux/idents/mots-clés/opérateurs ; pile d'indentation → `Indent`/`Dedent`/`Newline` ; profondeur `{}` → **suspension** off-side ; erreurs lexicales (tab/mélange, char illégal) au `Span`.

## Critère de done
Tokenise le programme V0 dans **les 2 syntaxes** ; tab/mélange → erreur au `Span` ; re-slice par `Span` = lexème ; bloc `{…}` multi-lignes sans `Indent`/`Dedent` ; tests verts ; build vert.

## Notes
Coins durs §1bis (`Newline` en accolades) : cf. risques 10100. Ne touche **que** `lex.rs`.
