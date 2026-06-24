# [4.1] Interop élargi (50400)

- Spec parente : 50400 · V6 · **Vague 4** · Statut : todo · Dépend de : 1.1
- (`bind.rs` **isolé** → parallélisable avec d'autres pistes.)

## But
Compléter l'interop : `bind` (types/chemins/versions), `rust:` indenté, types `Extern`.

## Fichiers possédés
`src/bind.rs` (mapping élargi + chemins sous-module), `src/lex.rs` (`rust:` indenté), `src/iri.rs`/`infer`/`lower` (`Extern` e2e).

## Étapes
1. `bind` : mapper `str`/`String`/collections/structs ; chemins `crate::mod::fn` via `.paths` ; ≥ v57.
2. `rust:` forme **indentée** (capture par indentation) + sauter `/* */`.
3. Type opaque `Extern` testé e2e (`extern type Inst = "std::time::Instant"`).

## Critère de done
`bind` d'une API `str`/struct → `.iri` utilisable ; `rust:` indenté compile ; type opaque e2e ; corpus 22/22 inchangé.

## Notes
Format rustdoc-json instable — cibler une version, dégrader proprement.
