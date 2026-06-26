# Iron — V9.1 / cadrage (`README.md`)

> **Version « langage pour développeurs ».** V9.1 prend Iron (qui *marche*) et le rend **agréable et
> professionnel à utiliser** : **zéro bibliothèque Iron** (tout passe par l'import de crates Rust), un
> **support langage** (diagnostics/complétion) bâti **par-dessus l'outillage Rust** (on transpile, rustc
> tranche, on remonte en Iron — la même surcouche que pour le reste du langage), et une **intégration
> VSCode** complète (fichiers `.irn`, coloration, LSP, linter, gestionnaire d'imports). Plus le **comblement
> des manques** restants (génériques, erreurs typées, paquets, doc, debug).
> Parent : `specs/generated.md` (V9.1 ajoutée). Contrat de contenu : `../generator.md`.
> Statut : **🔶 cadrage** (architecture + décisions ; specs détaillées + planif ci-jointes).
>
> **Toutes les specs ci-dessous sont planifiées (1:1 piste↔spec).** La durée de réalisation importe peu ; le
> but est d'avoir **une liste propre et exhaustive de ce qui reste**, qui ne dépende plus de la mémoire.

## Principe directeur (ce qui change la philosophie)

V9 a prouvé qu'Iron **n'a pas besoin d'une stdlib à lui** : il génère du Rust qui **appelle** des crates
compilés par cargo. V9.1 **assume ça à fond** : *aucune ligne de bibliothèque Iron n'est livrée.* Les seules
choses « intégrées » sont les **primitives de langage** (codegen pur : `print`→`println!`, `len`→`.len()`,
`range`→`0..n`) — ce n'est pas une lib, c'est le langage. **Tout le reste = `import` d'un crate Rust** (y
compris `std`). L'écosystème d'Iron **EST** crates.io.

## Carte des specs (5 chantiers, 23 specs — TOUTES planifiées)

### A — « Zéro lib Iron » : tout par import Rust (interop `5`)
| Spec | Contenu |
|---|---|
| `52000` | **Principe « aucune lib Iron »** : primitives = codegen ; le reste = import. Convertir/documenter les pseudo-modules `io`/`os`/`time` en **imports** (`import std` / crates) — plus de modules « stdlib » spéciaux |
| `52100` | **`import <crate>` auto-bind complet** : rustdoc-json à la volée + **cache** (`target/iron-bindings/`), résolution de version via `ironc.toml`. L'import universel, sans `bind`+`use` manuels |
| `52200` | **`import std` / `import std::collections`** : la **std Rust par import** (HashMap, fs, env, thread…) — `std` traité comme un crate bindé |
| `52300` | **Registre de bindings curés + `ironc add`** : `.iri` mutualisés des crates « bénis » (serde/regex/tokio/rand/clap…) = la **stdlib de facto** ; seed top-N |

### B — Support langage via overlay sur rustc (diagnostics `6`, tooling `7`)
| Spec | Contenu |
|---|---|
| `60800` | **Diagnostics haute-fidélité par overlay rustc** : transpile → `cargo check` → **remonte les erreurs rustc en `Span` Iron** (I8). La **surcouche** : pas de type-checker Iron maison (zéro faux-positif — c'est rustc qui tranche), exactement comme le codegen |
| `60900` | **Source-maps granularité expression** : mapping fin `.irn`↔`.rs` par expression (prérequis de la remontée précise B/60800) |
| `71300` | **LSP enrichi** : complétion (méthodes des types/crates bindés), hover (type inféré), go-to-def, rename — **plus** l'overlay rustc (60800) dans le serveur |
| `71400` | **`ironc fmt`** : formateur canonique **avec préservation des commentaires** (trivia lexer) |

### C — Intégration VSCode (tooling `7`)
| Spec | Contenu |
|---|---|
| `71500` | **Extension VSCode** : déclaration du langage `iron`, association `.irn`/`.iri`, **icônes de fichier** dans l'explorateur, configuration de base |
| `71600` | **Grammaire de coloration** (TextMate `.tmLanguage`) : mots-clés, types, chaînes, f-strings, commentaires, décorateurs `@` |
| `71700` | **Client LSP + linter dans VSCode** : connexion à `ironc lsp` → diagnostics/complétion/hover live ; les lints (`ironc lint`) + remontées rustc (60800) dans le panneau **Problems** |
| `71800` | **Gestionnaire d'imports VSCode** : quick-fix « importer `regex` », commande `ironc add <crate>` intégrée, auto-import des méthodes de crates bindés |
| `71900` | **Build/run intégrés** : tasks VSCode (`ironc build/run/test`), **build-on-save**, exécution au clic, codelens |

### D — Profondeur & maturité (types `2`, tooling `7`)
| Spec | Contenu |
|---|---|
| `21400` | **Génériques utilisateur complets** : `def f[T](x: T) -> T`, `class Box[T]`, bornes (`[T: Display]`) → génériques/bornes Rust |
| `21500` | **Erreurs typées `Result[T, E]`** : lever I13 (au-delà de l'`IronError` unifié) — enums d'erreur, `E` concret |
| `21600` | **Closures avancées** : closures **stockées** (champ de struct), **retournées** (`-> fn(...)`), captures complexes (`Box<dyn Fn>`) |
| `72000` | **`ironc repl`** : éval incrémental (compile-and-run par snippet) |
| `72100` | **Gestion de paquets** : `ironc.toml` + **lockfile** + résolution de versions (au-delà de l'injection brute) |
| `72200` | **`ironc doc`** : génération de doc Iron (overlay sur rustdoc, comme le reste) |
| `72300` | **Intégration debugger** : gdb/lldb montrant les **lignes Iron** via les source-maps (I8) |

### E — Le « 0.1 % » & validation (méta `9`)
| Spec | Contenu |
|---|---|
| `90600` | **Le « 0.1 % » tracé** : inférence Hindley-Milner complète · **auto-hébergement** · async exotique (select/pin) · binding **garanti** de tout crate — **explicitement hors-périmètre**, documenté pour ne pas l'oublier |
| `90700` | **Batterie élargie** : 30 vrais crates (texte/web/données/concurrence) — finir la batterie V9/6.1 |
| `90800` | **Gate V9.1 final** : projet-vitrine + extension VSCode fonctionnelle + tous les outils ; verdict honnête |

## Décisions structurantes (à FIGER en phase 1)

- **✅ D1 — Aucune bibliothèque Iron.** On ne livre **aucun** code de lib Iron. Primitives de langage = codegen
  pur. Tout le reste (collections avancées, IO, temps, réseau, JSON, regex…) = **import d'un crate Rust** (ou
  `import std`). Les pseudo-modules `io`/`os`/`time` de V7 sont **convertis** en imports (52000). L'écosystème
  = crates.io. *Conséquence : pas de spec stdlib, jamais.*
- **✅ D2 — Support langage = overlay sur rustc (la surcouche).** Les diagnostics riches ne viennent **pas**
  d'un type-checker Iron réécrit (source de faux-positifs), mais de la **transpilation → `cargo check` →
  remontée des erreurs rustc en `Span` Iron** (60800). Exactement le modèle du codegen (I2/I8) : Iron pose la
  surcouche, **rustc fait le travail dur**. Le LSP enrichi (71300) consomme cet overlay + des analyses Iron
  natives (complétion/hover) pour la réactivité.
- **✅ D3 — VSCode est la cible d'intégration de référence.** L'extension (C) est le livrable « expérience
  dev » : fichiers reconnus, colorés, diagnostiqués, importables, buildables. Protocole standard (LSP) → ré-
  utilisable par tout éditeur, mais VSCode est ce qu'on **prouve**.
- **🔶 D4 — Réactivité du LSP (à arbitrer).** `cargo check` (overlay) est **lent** (secondes). Le LSP doit
  combiner : analyse Iron **instantanée** (lex/parse/inférence locale → diagnostics rapides) + overlay rustc
  **asynchrone/débattu** (erreurs de type profondes, en arrière-plan). **Proposé** : deux niveaux de
  diagnostics, le rapide d'abord, le profond ensuite.
- **🔶 D5 — Erreurs typées (21500) : ampleur.** `Result[T, E]` avec `E` concret touche l'inférence de flux
  (I11) et le désucrage `?`/`try`. **Proposé** : `E` par défaut `IronError` (rétro-compat), `E` explicite
  opt-in via annotation. À arbitrer (peut être lourd).

## Priorité / ordre des chantiers

1. **A — zéro lib / import universel** (fonde « tout par Rust » ; débloque le reste de l'écosystème).
2. **B — support langage** (overlay rustc + LSP enrichi + fmt ; prérequis de l'intégration VSCode).
3. **C — VSCode** (consomme B ; l'expérience dev visible).
4. **D — profondeur & maturité** (génériques, erreurs typées, closures, repl, paquets, doc, debug ;
   parallélisable, sous-systèmes disjoints).
5. **E — 0.1 % tracé + batterie + gate V9.1**.

## e2e / gate V9.1

Un dev ouvre un projet Iron dans **VSCode** : fichiers `.irn` reconnus (icônes), colorés, **diagnostics
live** (rapides + overlay rustc), **complétion** des méthodes de crates importés, **quick-fix d'import** de
crate, **build/run au clic**, `ironc fmt` au save. Le projet **n'utilise aucune lib Iron** — uniquement des
`import` de crates Rust (+ `import std`). `ironc doc` génère la doc ; un breakpoint s'arrête sur une **ligne
Iron**. Métrique de vérité ≥ celle de V9 ; **0 `unsafe`** ; tous les `🔶` tranchés.

## Hors V9.1 (le « 0.1 % », tracé en `90600`)

HM complet · auto-hébergement · async exotique · binding **garanti** de n'importe quel crate. Tout le reste
**est dans une spec ci-dessus**, et **toutes** sont planifiées.
