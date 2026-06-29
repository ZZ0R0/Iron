# Référence CLI `ironc`

Binaire : `ironc/target/release/ironc` (après `cargo build --release --manifest-path ironc/Cargo.toml`).
Forme générale : `ironc <commande> [flags] <fichier.irn | crate>`.

Sans fichier, les commandes projet (`build/run/emit/check/test`) utilisent `[iron] entry` de `ironc.toml`.

## Commandes

| Commande | Effet |
|---|---|
| `ironc run <f.irn>` | transpile → `cargo run` (compile + exécute le Rust généré). |
| `ironc build <f.irn>` | transpile → `cargo build` (vérifie que ça compile). |
| `ironc build --debug <f.irn>` | build debug + **source-map** `sourcemap.json` + `launch.json` (CodeLLDB, I8). |
| `ironc emit <f.irn>` | écrit le **Rust lisible** dans `<stem>-rust/` **sans compiler** (lis/édite le `.rs`). |
| `ironc emit --no-std <f.irn>` | émet une **lib `#![no_std]`** + `alloc` (embarqué). |
| `ironc check <f.irn>` | diagnostics haute-fidélité par overlay `cargo check` (erreurs `rustc` → ligne Iron). |
| `ironc test <f.irn>` | `cargo test` (les `def test_*()` deviennent des `#[test]`). |
| `ironc fmt <f.irn>` | formate la source **en place** (préserve les commentaires). |
| `ironc lint <f.irn>` | passe de lint (warnings, jamais bloquant). |
| `ironc new <nom>` | scaffolding `<nom>/{src/main.irn, ironc.toml, .gitignore}`. |
| `ironc add <crate>` | binde un crate → `<crate>.gen.iri` + `<crate>.iri.report.md` + l'ajoute à `ironc.toml`. |
| `ironc add <crate> --strict` | échoue si la couverture < seuil (90 %). |
| `ironc remove <crate>` | retire le crate (`ironc.toml` + `.gen.iri`/`.iri`/rapport). |
| `ironc update` | re-binde tous les `<crate>.gen.iri` du dossier (curation `.iri` préservée). |
| `ironc doc <f.irn>` | `cargo doc` via overlay (doc rustdoc des items publics Iron). |
| `ironc lib <f.irn>` | exporte un crate Rust réutilisable (inverse de l'interop). |
| `ironc lsp` | serveur LSP (stdio) — diagnostics live dans l'éditeur. |
| `ironc repl` | REPL interactif. |

## Flags

| Flag | Sur | Effet |
|---|---|---|
| `--audit` | build/run | affiche les **warnings de perf** (clone-fallback, I4) + couverture du moteur d'inférence. |
| `--debug` | build | build debug + source-maps + launch.json (I8). |
| `--instrumented` | emit | garde les marqueurs de source-map (sinon Rust « propre »). |
| `--no-std` | emit | émet `#![no_std]` + `alloc` (lib embarquée). |
| `--strict` | add | échec si couverture binding < 90 %. |

## `ironc.toml` (le « Cargo.toml » d'Iron)

```toml
[package]
name = "mon_projet"
version = "0.1.0"

[iron]
entry = "src/main.irn"      # fichier d'entrée par défaut (run/build sans argument)
emit = "clean"             # "clean" (lisible) | "instrumented" (source-maps)
rust-out = "rust"          # dossier où `ironc emit` écrit le Rust (défaut <stem>-rust/)

[dependencies]
serde_json = "*"           # rempli par `ironc add`
```

## Dossiers temporaires

- `ironc run/build/check` : projet cargo dans `/tmp/ironc-out/<stem>_<hash>/` (hash du chemin → projets parallèles distincts).
- `ironc add` : projet de binding dans `/tmp/ironc-add-<crate>/` (compile le crate pour rustdoc).
- `ironc emit` : `<stem>-rust/` à côté du source (**visible**, pour lire/éditer le Rust).

## Pré-requis

- **Rust stable** : tout sauf `add`.
- **Rust nightly** : `ironc add` (rustdoc JSON). `rustup toolchain install nightly`.
