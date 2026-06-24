# [1.1] Fondation : squelette e2e + types IR gelés

- Spec(s) parente(s) : **toutes** (fondation transverse)
- Version : V0 · **Vague 1** (séquentielle) · Statut : **done ✅** (gate vert, 2026-06-24)
- Dépend de : —

## But
Colonne e2e fine qui **boote** : `ironc run trivial.irn` → `cargo` vert → s'exécute. Geler les types IR en Rust, câbler tous les passes en **stubs**, et **prouver le mécanisme source-map** (dérisquage n°1) au trivial.

## Fichiers possédés
Workspace `ironc/` (`Cargo.toml`) · `src/main.rs` (CLI minimal) · `src/ir.rs` (**tous** les types gelés, README §3–4) · `src/{lex,parse,resolve,lower,sourcemap,driver}.rs` + `src/infer/mod.rs` (**stubs** signés) · `tests/fixtures/`.

## Interface produite (gelée pour la vague 2)
- `ir.rs` : `Span, FileId, NodeId, Token, TokenKind, Ast(+nœuds §4.2), Type, TypeMap, OwnershipMap, Disposition, SourceMap (+ format de marqueur)`.
- Signatures : `lex(&str, FileId)->Vec<Token>` · `parse(Vec<Token>)->Result<Ast>` · `infer(&Ast)->TypeMap` · `resolve(&Ast,&TypeMap)->OwnershipMap` · `lower(&Ast,&TypeMap,&OwnershipMap)->(TokenStream, SourceMap)` · driver `build/run`.

## Étapes
1. Workspace + deps (`proc_macro2`, `quote`, `prettyplease`, `syn` au besoin).
2. `ir.rs` : tous les types (README §3–4) ; le crate **généré** porte `#![forbid(unsafe_code)]` + `IronError`.
3. Stubs des passes (retours triviaux/vides, **types corrects**).
4. Driver : `ironc build/run` → passes stub → `main.rs` minimal → `prettyplease` → `cargo`.
5. **Preuve source-map** : `lower` émet 1 marqueur lié à un `NodeId` ; `sourcemap` le relocalise après `prettyplease` ; `assert` du mapping.
6. ≥1 fixture partagée (`source→tokens→ast→typemap→ownmap` trivial).

## Critère de done (vérifiable)
- `cargo build` du compilateur **vert**.
- `ironc run trivial.irn` → `cargo` **vert** → s'exécute.
- Le marqueur trivial **round-trip** (test assertant `Span_irn ↔ pos Rust`).
- `ir.rs` expose tous les types §3–4.

## Notes
**Gèle `ir.rs`** : aucune vague 2 ne le modifie. Tout trou d'interface se corrige **ici**, avant le fan-out.
