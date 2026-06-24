# [2.1] Inférence Option/Result par le flux + `try/except` (20400)

- Spec parente : 20400 · V6 · **Vague 2** · Statut : todo · Dépend de : 1.1
- **⚠ RISQUE N°1 de V6** (peut mé-typer des fns du corpus).

## But
Honorer **I11** : déduire `Option`/`Result` du flux de contrôle ; `try/except` → `match`.

## Fichiers possédés
`src/infer/flow.rs`, `src/lower.rs` (return wrapping), `src/parse.rs` (try/except).

## Étapes
1. `definitely_returns(stmts)` (if/match exhaustifs, return) — **doit** classer `match` tous-bras-`return` comme retournant.
2. Fn qui **chute** + `return <valeur>` → ret effectif `Option<T>` ; `?` → `Result`.
3. `lower` : signature effective, `return v`→`Some/Ok`, chute→`None`, `sigs` mis à jour (appels).
4. `try: … except e: …` → `match` sur le `Result`.

## Critère de done
fn à chute → `Option` exécutable ; `?` → `Result` ; `try/except` exécute ; **corpus 22/22 inchangé** (test-gardé).

## Notes
Si une fn du corpus est reclassée à tort → l'analyse `definitely_returns` est fausse, corriger avant de continuer.
