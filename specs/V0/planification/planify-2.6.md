# [2.6] Source maps + remontée rustc (60100) — ⚠ PRIORITÉ

- Spec parente : 60100 · V0 · **Vague 2** (parallèle) · Statut : todo
- Dépend de : 1.1 (mécanisme de marqueur prouvé)
- **À DÉMARRER EN PREMIER dans la vague 2 — risque n°1 (`generator §8`).**

## But
Remplir `src/sourcemap.rs` : relocaliser les marqueurs **après `prettyplease`** (`Span_irn ↔ Rust`, I8) + **remonter** les diagnostics `cargo`/`rustc` au `Span` Iron.

## Fichiers possédés
`src/sourcemap.rs` (+ `tests/sourcemap_*.rs`).

## Interface
Consomme `SourceMap` (de `lower`) + sortie `cargo` JSON (de `driver`). Produit le mapping + les diagnostics remontés.

## Étapes
Ré-localisation des marqueurs dans le Rust formaté → table `Span_irn ↔ (ligne,col)` ; parsing diagnostics `cargo --message-format=json` ; re-pointage `Span` Iron ; granularité **instruction** d'abord, **expression** ensuite.

## Critère de done
Erreur `rustc` forcée → message pointant la **bonne ligne Iron** (pas du Rust) ; mapping **robuste au reformatage** (un re-`prettyplease` ne casse pas le pointage) ; tests verts ; build vert.

## Notes
Repli **granularité instruction** et/ou mécanisme type `#[line]` si les marqueurs ne survivent pas proprement. Ne touche **que** `sourcemap.rs`.
