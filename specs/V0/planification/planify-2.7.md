# [2.7] Driver cargo + CLI ironc (70100)

- Spec parente : 70100 · V0 · **Vague 2** (parallèle) · Statut : todo
- Dépend de : 1.1 (squelette driver + types gelés)

## But
Remplir `src/driver.rs` + `src/main.rs` : `ironc build/run` réel — `TokenStream` → projet cargo (`prettyplease`) → `cargo`, capture de la sortie.

## Fichiers possédés
`src/driver.rs`, `src/main.rs` (+ `tests/driver_*.rs`).

## Interface
Consomme `(TokenStream, SourceMap)` (de `lower`). Produit le projet cargo, relaie exécution + diagnostics JSON (→ 2.6).

## Étapes
Génération `Cargo.toml` + `src/main.rs` (pretty-printé) ; invocation `cargo build/run` ; capture `--message-format=json` ; code de retour relayé ; mono-fichier (§4.7).

## Critère de done
`ironc run` sur le programme V0 → `cargo` **vert** → **sortie correcte** ; projet reproductible (même entrée → mêmes fichiers, modulo `prettyplease`) ; tests verts ; build vert.

## Notes
Emplacement/cache du projet généré + toolchain ciblée : figer (cf. risques 70100). Ne touche **que** `driver.rs`/`main.rs`.
