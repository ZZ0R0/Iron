# [1.1] Fondation V1 : contrat d'emprunt gelé, corpus vert en all-clone

- Spec parente : 30200 · V1 · **Vague 1** (séquentielle) · Statut : todo
- Dépend de : V0 (gelé)

## But
Étendre le contrat sans rien casser : `ir.rs` gagne `Disposition::Borrow` + `ParamModes`/`ParamMode`.
`resolve` renvoie **all-Clone** (comportement V0) ; `lower` **sait gérer** `ByRef`/`Move`/`Borrow` (chemins
câblés mais non encore sollicités). Brancher le flag CLI `--audit`. **Le corpus reste vert, identique à V0.**

## Fichiers possédés
`src/ir.rs` (extension), `src/resolve.rs` (all-Clone + `ParamModes` tous `ByValue`), `src/lower.rs`
(consomme `ParamModes`/`Disposition`), `src/main.rs` + `src/driver.rs` (flag `--audit`).

## Interface produite (gelée)
- `Disposition = { Clone, Move, Borrow }`.
- `ParamMode = { ByValue, ByRef }` ; `ParamModes : fn_name → Vec<ParamMode>`.
- `resolve(&Ast, &TypeMap) -> (OwnershipMap, ParamModes)`.

## Critère de done
`cargo test` vert ; les **11 programmes du corpus** sortent **exactement** comme en V0 ; `ir.rs` expose
`Borrow`/`ParamModes` ; `--audit` reconnu (no-op tant que pas de clone signalé).

## Notes
All-`ByValue` + all-`Clone` ⇒ Rust identique à V0 ⇒ non-régression triviale avant le fan-out.
