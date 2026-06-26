# V9.1 — Planification (vagues & pistes)

> **Toutes les specs sont planifiées (1:1 piste↔spec).** 5 vagues séquentielles, 23 pistes. Gabarit :
> `generator §6`. Parente : `../README.md`. La durée de réalisation importe peu — le but est une **liste
> propre et exhaustive** de ce qui reste, qui ne dépende plus de la mémoire.

## Table de correspondance piste → spec (23 pistes, 23 specs)

| Piste | Spec | Titre | Vague |
|---|---|---|---|
| 1.1 | 52000 | Principe « aucune lib Iron » + retrait io/os/time | 1 |
| 1.2 | 52100 | `import <crate>` auto-bind complet (rustdoc + cache) | 1 |
| 1.3 | 52200 | `import std` (std Rust par import) | 1 |
| 1.4 | 52300 | Registre curé + `ironc add` (stdlib de facto) | 1 |
| 2.1 | 60800 | Diagnostics overlay rustc (la surcouche) | 2 |
| 2.2 | 60900 | Source-maps granularité expression | 2 |
| 2.3 | 71300 | LSP enrichi (complétion/hover/go-to-def) | 2 |
| 2.4 | 71400 | `ironc fmt` (préservation commentaires) | 2 |
| 3.1 | 71500 | Extension VSCode : langage `.irn`/`.iri` + icônes | 3 |
| 3.2 | 71600 | Grammaire de coloration (TextMate) | 3 |
| 3.3 | 71700 | Client LSP + linter dans VSCode (Problems) | 3 |
| 3.4 | 71800 | Gestionnaire d'imports VSCode | 3 |
| 3.5 | 71900 | Build/run intégrés VSCode | 3 |
| 4.1 | 21400 | Génériques utilisateur complets | 4 |
| 4.2 | 21500 | Erreurs typées `Result[T, E]` | 4 |
| 4.3 | 21600 | Closures avancées (stockées/retournées) | 4 |
| 4.4 | 72000 | `ironc repl` | 4 |
| 4.5 | 72100 | Gestion de paquets (lockfile, versions) | 4 |
| 4.6 | 72200 | `ironc doc` | 4 |
| 4.7 | 72300 | Intégration debugger (lignes Iron) | 4 |
| 5.1 | 90600 | Le « 0.1 % » tracé (hors-périmètre) | 5 |
| 5.2 | 90700 | Batterie élargie 30 vrais crates | 5 |
| **5.3** | 90800 | **Gate V9.1 final** | 5 |

## Parallélisme (test mental anti-collision)

- **Vague 1 (interop)** : `bind.rs`/`driver` partagés → séquentiel-incrémental (1.1 retrait d'abord, puis
  import/std/registre). `1.4` (dossier `bindings/`) parallélisable.
- **Vague 2 (support langage)** : `60900` (source-maps, `lower`/`sourcemap`) **d'abord** (prérequis) → `60800`
  (overlay, `driver`) → `71300` (LSP). `71400` (fmt, `lex`/`fmt.rs`) parallélisable (ownership disjoint).
- **Vague 3 (VSCode)** : **nouveau sous-arbre `extensions/vscode/`** (TypeScript/JSON) → **disjoint du
  compilateur Rust** → fan-out possible ; `3.3` dépend du LSP (2.3), `3.4` du registre (1.4).
- **Vague 4 (profondeur)** : `21400`/`21500`/`21600` touchent `infer`/`lower` → séquentiel ; `72000`/`72100`/
  `72200`/`72300` (tooling/driver) parallélisables entre eux.
- **Vague 5 (validation)** : `5.1` (doc) ∥ `5.2` (batterie) → `5.3` (gate, agrège tout).

## Ordre d'exécution

1 (zéro-lib/import) → 2 (support langage : source-maps → overlay → LSP ∥ fmt) → 3 (VSCode, sur LSP) →
4 (profondeur & maturité) → 5 (0.1 % tracé ∥ batterie → gate V9.1).

## Gate V9.1 (= done de 5.3)

Projet-vitrine **sans aucune lib Iron** (que des `import` Rust + `import std`), génériques/erreurs-typées/
closures, concurrence → compile+run (0 `unsafe`). **VSCode** : `.irn` reconnus+icônes, colorés, diagnostics
live (rapides + overlay rustc), complétion des crates importés, quick-fix d'import, build/run au clic, fmt au
save ; breakpoint sur ligne Iron. `check/fmt/lint/doc/repl/lib/add` OK. Corpus + batterie-30 verts ; métrique
≥ V9 ; **tous les `🔶` tranchés**. Reste seulement le « 0.1 % » (90600), tracé.
