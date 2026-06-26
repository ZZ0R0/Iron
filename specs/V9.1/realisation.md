# V9.1 — Réalisation (tracker)

> **État : cadrage + planification FAITS. Réalisation NON commencée** (attend le feu vert). **Liste propre et
> exhaustive** de ce qui reste — ne dépend plus de la mémoire. 23 pistes, 1:1 avec les specs.

## Avancement des pistes (5 vagues, 23 pistes)

| Piste | Spec | Titre | Vague | Statut |
|---|---|---|---|---|
| 1.1 | 52000 | Principe « aucune lib Iron » + retrait io/os/time | 1 | ☐ todo |
| 1.2 | 52100 | `import <crate>` auto-bind complet (rustdoc + cache) | 1 | ☐ todo |
| 1.3 | 52200 | `import std` | 1 | ☐ todo |
| 1.4 | 52300 | Registre curé + `ironc add` | 1 | ☐ todo |
| 2.1 | 60800 | Diagnostics overlay rustc | 2 | ☐ todo |
| 2.2 | 60900 | Source-maps granularité expression | 2 | ☐ todo |
| 2.3 | 71300 | LSP enrichi (complétion/hover/go-to-def) | 2 | ☐ todo |
| 2.4 | 71400 | `ironc fmt` (commentaires) | 2 | ☐ todo |
| 3.1 | 71500 | Extension VSCode : langage + icônes | 3 | ☐ todo |
| 3.2 | 71600 | Grammaire de coloration | 3 | ☐ todo |
| 3.3 | 71700 | Client LSP + linter VSCode | 3 | ☐ todo |
| 3.4 | 71800 | Gestionnaire d'imports VSCode | 3 | ☐ todo |
| 3.5 | 71900 | Build/run intégrés VSCode | 3 | ☐ todo |
| 4.1 | 21400 | Génériques utilisateur complets | 4 | ☐ todo |
| 4.2 | 21500 | Erreurs typées `Result[T, E]` | 4 | ☐ todo |
| 4.3 | 21600 | Closures avancées | 4 | ☐ todo |
| 4.4 | 72000 | `ironc repl` | 4 | ☐ todo |
| 4.5 | 72100 | Gestion de paquets | 4 | ☐ todo |
| 4.6 | 72200 | `ironc doc` | 4 | ☐ todo |
| 4.7 | 72300 | Intégration debugger | 4 | ☐ todo |
| 5.1 | 90600 | Le « 0.1 % » tracé | 5 | ☐ todo |
| 5.2 | 90700 | Batterie élargie 30 vrais crates | 5 | ☐ todo |
| **5.3** | 90800 | **Gate V9.1 final** | 5 | ☐ todo |

## Principe figé (D1) — ne jamais l'oublier

**Aucune bibliothèque Iron.** Primitives de langage = codegen ; tout le reste = `import` d'un crate Rust (ou
`import std`). L'écosystème d'Iron EST crates.io. *Il n'y aura jamais de spec stdlib.*

## Méthode figée (D2) — support langage

Les diagnostics riches viennent de **rustc via overlay** (transpile → `cargo check` → remontée en `Span` Iron),
pas d'un type-checker Iron maison. La surcouche, comme le codegen.

## Écarts / faiblesses (à remplir en phase 3-4)

_(rien encore — réalisation non commencée)_

## Hors V9.1 (le « 0.1 % », spec 90600)

HM complet · auto-hébergement · async exotique · coroutines complexes · binding **garanti** de tout crate.
Tout le reste est dans une spec ci-dessus, **toutes planifiées**.
