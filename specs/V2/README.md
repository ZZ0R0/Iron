# Iron — V2 / cadrage (`README.md`)

> **Fondation V2 = interop Rust par niveaux (I7).** `50100` `rust:` inline → `50200` bindings `.iri` →
> `50300` auto-gen rustdoc-json. I7 : **ne pas implémenter un niveau avant le précédent**.
> Parent : `specs/generated.md`. Contrat : `../generator.md`. Statut : **🔶 proposé** — trancher §5.

## 1. Périmètre proposé

- **Niveau 1 (`50100`) — `rust:` inline** : échappatoire bas niveau, du Rust **verbatim** dans une fn Iron.
- **Niveau 2 (`50200`) — bindings `.iri`** : déclarer des items Rust externes (fn/types) + `use` + dépendance
  crate ; appels **type-checkés** côté Iron.
- 🔶 **Niveau 3 (`50300`) — auto-gen rustdoc-json : proposé HORS V2.** C'est le plus dur (parsing d'un format
  externe volumineux et instable entre versions de `rustc`). **V2 = niveaux 1+2** (interop réelle et utilisable) ;
  le niveau 3 = increment/version ultérieure.

## 2. Niveau 1 — `rust:` inline (`50100`)

- **Syntaxe** : un bloc `rust:` (indenté **ou** `{}`, §1bis V0) dont le contenu est **émis verbatim** dans le
  Rust généré, à sa place.
- **Pont de valeurs** : les variables Iron en portée sont **accessibles par leur nom** (elles lowerent déjà vers
  des vars Rust de même nom). Le bloc lit/écrit ces noms ; en position d'expression, sa valeur Rust est sa valeur.
- **Responsabilité de l'auteur** (I7 niveau 1 = bas niveau) : connaître les types Rust générés — y compris les
  **emprunts `&T`** des params (V1). Le contenu n'est **pas** vérifié côté Iron ; `rustc` = filet (I2), remontée I8.
- **Garde-fou I2 automatique** : `#![forbid(unsafe_code)]` reste en tête de crate → un `unsafe` glissé dans un
  `rust:` **ne compile pas**. Iron ne peut pas émettre d'`unsafe`, même via l'échappatoire.

## 3. Niveau 2 — bindings `.iri` (`50200`)

- Un `.iri` déclare des **signatures externes** : `extern fn name(params) -> ret = "rust::path"` et des **types
  opaques** `extern type Name = "rust::Path"`, plus d'éventuelles **dépendances** `crate nom = "version"`.
- **Import** : `use "math.iri"` en tête d'un `.irn` → les items deviennent appelables, **type-checkés** contre la
  signature déclarée, lowerés vers le `rust::path`.
- **Dépendance crate** : le driver alimente le `[dependencies]` du `Cargo.toml` généré (aujourd'hui vide) à partir
  des `crate …` des `.iri` importés.
- **Type opaque** : vu côté Iron comme un nouveau `Type::Extern(name)` — boîte noire, manipulable seulement via
  les fns du binding (sémantique valeur I1 : clone/move comme les autres).

## 4. Conventions figées

- Pont de noms (niveau 1) : var Iron `x` ↔ var Rust `x` (déjà vrai V0/V1).
- `.iri` = format texte déclaratif (signatures + types + crates) ; gabarit en planif.
- Nouveau IR : `Type::Extern(String)` (extension rétro-compatible) ; nœud AST `RustBlock(String)` (verbatim).
- `Cargo.toml` généré gagne une section `[dependencies]` alimentée par les `.iri`.

## 5. Décisions ouvertes (fond — à trancher)

- 🔶 **Scope V2** : niveaux **1+2** (recommandé), niveau 3 (rustdoc-json) **reporté**. → ok, ou tu veux les 3 ?
- 🔶 **Syntaxe d'import** : `use "math.iri"` (chemin fichier, simple) vs `import math` (résolution implicite).
  → `use "…"` ?
- ❓ **Pont de valeurs niveau 1** : accès **par nom** (verbatim, bas niveau) — confirmé ? (vs interface
  explicite entrées/sorties, plus lourde mais plus sûre).

## 6. Critères d'acceptation V2 (gate)

1. **Niveau 1** : une fn Iron avec un bloc `rust:` appelant du Rust réel (p. ex. `.sqrt()` sur un `f64`) →
   compile + s'exécute correctement.
2. **Niveau 2** : un `.iri` (binding sur `std` ou une crate réelle) importé via `use`, appelé depuis Iron,
   **type-checké** → compile + exécute ; la dépendance apparaît dans le `Cargo.toml` généré.
3. **Zéro `unsafe`** émis (I2) ; un `unsafe` dans un `rust:` est **rejeté** (forbid).
4. **Corpus V0/V1 toujours vert** (non-régression).

## 7. Suite (après validation)

Fan-out : `50100` (`rust:` inline) **puis** `50200` (`.iri` + `use` + dépendance crate), ordre I7. Détail en phase 2.
