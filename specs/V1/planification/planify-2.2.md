# [2.2] Lowering emprunts + `--audit`

- Spec parente : 30200 · V1 · **Vague 2** (parallèle) · Statut : todo
- Dépend de : 1.1 (contrat gelé ; testable sur `ParamModes` fixtures)

## But
`lower.rs` consomme `ParamModes` + `Disposition` : émettre `&T`/`&x`, `move`, `borrow`, et le warning `--audit`.

## Fichiers possédés
`src/lower.rs` (+ `tests/lower_*.rs`).

## Interface
Consomme `&Ast, &TypeMap, &OwnershipMap, &ParamModes`. Produit `(TokenStream, SourceMap)` + warnings audit.

## Étapes
- Signatures : param `ByRef` → `#name: &#ty` ; déréf transparent dans le corps (l'accès champ/index marche
  sur `&T`).
- Call sites : argument vers un param `ByRef` → `&(#arg)` ; vers `ByValue` → valeur (move/clone selon map).
- Lecture locale : `Move` → `#id` ; `Borrow` → `&#id` ; `Clone` → `#id.clone()` (+ collecte un warning audit).
- `--audit` : si activé, émettre les warnings *perf* (span Iron) ; sinon silencieux (I4).

## Critère de done
Rust généré du corpus : **strictement moins de `.clone()`** qu'en V0 ; compile ; `--audit` liste les fallbacks.
Build vert.

## Notes
Ne change PAS la sémantique : un `&T` lu donne les mêmes valeurs. Vérifié réellement en 3.1.
