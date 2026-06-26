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

## Avancement (passe 1 de réalisation) — corpus + binding VERTS, 0 unsafe

**✅ FAIT** :
- **Vague 1 COMPLÈTE** (zéro lib Iron) : `import std` (registre curé embarqué) · retrait io/os/time ·
  `ironc add <crate>` (rand → 19 bindings → `import rand`) · résolution d'import à niveaux · robustesse parser
  `.iri` (tuples/unit). Principe figé dans `generator.md` (I7bis).
- **2.1** `ironc check` : diagnostics overlay rustc (la surcouche) — erreur rustc → Span Iron, zéro faux-positif.
- **Vague 3 COMPLÈTE** (extension VSCode `editors/vscode/`) : langage `.irn`/`.iri` + icône · coloration
  TextMate · client LSP + `ironc check` au save · `Iron: Add crate` · build/run + CodeLens « ▶ Run ».
- **4.1** génériques utilisateur (`def f[T]`/`class Pair[A,B]` + bornes ; borne `Clone` par défaut) → 7, 30.
- **4.6** `ironc doc` (overlay rustdoc → HTML).
- **2.2** source-maps : la ligne marche déjà (granularité expression = raffinement non fait).

**☐ RESTE** : 2.3 LSP enrichi (complétion/hover — diagnostics OK) · 2.4 fmt (bloqueur trivia commentaires) ·
4.2 erreurs typées `Result[T,E]` (D5, lourd) · 4.3 closures avancées · 4.4 repl · 4.5 paquets (lockfile :
Cargo.lock déjà auto ; reste remove/update) · 4.7 debugger · Vague 5 (5.1 doc 0.1% · 5.2 batterie-30 · 5.3 gate).

## Écarts / faiblesses (relevés)

- L'auto-bind produit des types (tuples/unit) que le parser `.iri` ne relisait pas → corrigé (cohérence
  bind↔parse à surveiller pour d'autres constructs).
- io/os/time **retirés** : un programme les utilisant doit migrer vers `import std`. Aucun test ne les
  utilisait (migration nulle).

## Hors V9.1 (le « 0.1 % », spec 90600)

HM complet · auto-hébergement · async exotique · coroutines complexes · binding **garanti** de tout crate.
Tout le reste est dans une spec ci-dessus, **toutes planifiées**.
