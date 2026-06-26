# Iron — V9 / cadrage (`README.md`)

> **Version de CLÔTURE — « langage fini à 99.9% ».** V9 ferme **tous** les chantiers ouverts : interop à
> 100% (crates importables nativement, dans les deux sens), **concurrence** (threading + async), **polish du
> cœur** (les bords rugueux restants), **tooling complet** (fmt, LSP, REPL, `import`/`add`), **diagnostics &
> lint** réels, et la **métrique de vérité chiffrée**. À la fin de V9, Iron est un langage **utilisable pour
> de vrai, de bout en bout, avec son écosystème** — pas un prototype.
> Parent : `specs/generated.md` (V9 ajoutée). Contrat de contenu : `../generator.md`.
> Statut : **🔶 cadrage** (architecture + décisions ; specs détaillées + planif ci-jointes).
>
> **C'est la plus grosse version après V7.** 6 chantiers, ~21 specs. Tout ce qui était « différé V8+ » ou
> « hors-périmètre » dans V0→V8 **atterrit ici** et **doit être fait**.

## Pourquoi V9 (l'état réel à combler)

V8 a prouvé l'auto-binding sur **10 vrais crates** (regex, serde_json, csv, ureq/HTTP, std::thread, sha2,
chrono…) — mais a révélé **deux modes** :
- **Iron pur** (regex, serde_json) : API à méthodes simples + données possédées → syntaxe 100% Python.
- **`rust{}` hybride** (ureq, std::thread, sha2, chrono, csv) : API à **traits/builders/itérateurs/async** →
  on descend en Rust brut. C'est le **résidu** mesuré (couverture auto 62–92%).

V9 **élimine** ce résidu (sauf le 0.1% irréductible) et ferme tous les autres manques accumulés (tooling,
lint, diagnostics fins, concurrence, métrique chiffrée). **Objectif chiffré : un dev écrit un programme Iron
réel multi-fichiers, importe des crates en une ligne, utilise threading/HTTP/JSON/regex en syntaxe Python,
avec fmt/lint/LSP, et la correction est chiffrée sur corpus.**

## Carte des specs (6 chantiers, ~21 specs)

### A — Interop 100% (interop `5`, tooling `7`)
| Spec | Contenu |
|---|---|
| `51600` | **⭐ Binding des méthodes de traits** : `impl Trait for T` (Iterator/Display/From/`Add`…/traits user) + **protocoles bénis** → protocoles Iron (`for`, `str()`, `+`, `==`, `[]`). **LE levier** : fait passer sha2/chrono/itérateurs de `rust{}` → **Iron pur** |
| `51700` | **Robustesse du bind** : résolution des **ré-exports publics**, collisions de noms courts (`bytes::Regex` vs `Regex`), getter `&str`→`.to_string()`, itérateur d'`Item` non-scalaire, profondeur de modules |
| `51800` | **Inverse : Iron → crate Rust** (`ironc lib`) : émet un crate `{Cargo.toml, lib.rs}` réutilisable depuis un projet Rust |
| `70700` | **`import <crate>`** : auto-bind **caché + caché en cache**, version résolue via `ironc.toml` (1 ligne au lieu de `bind`+`use`) |
| `70800` | **Registre de bindings curés** + **`ironc add <crate>`** : `.iri` mutualisés (modèle @types/nimble), seed du top-N par l'équipe cœur |

### B — Concurrence (ownership `3`, tooling `7`, types `2`)
| Spec | Contenu |
|---|---|
| `30700` | **`shared_sync[T]` → `Arc<Mutex<T>>`** : partage mutable **thread-safe** (`Send`/`Sync`) — pendant `Send` du `shared`/`Rc` mono-thread (I5) |
| `70900` | **Module `thread`** : `spawn`/`join` (wrappers `std::thread`) + closures `move` **Send-aware** ; `rust{}` reste l'échappatoire |
| `21200` | **`async`/`await`** : `async def`→`async fn`, `await`→`.await`, inférence des `Future`, runtime via crate bindé (tokio/smol). **Le chantier le plus lourd** |

### C — Polish du langage cœur (lowering `4`, frontend `1`, types `2`)
| Spec | Contenu |
|---|---|
| `40300` | **Fix `try/except` + hoisting** : variable assignée dans un `try` lisible après (plus de capture non-initialisée de l'IIFE) |
| `40400` | **Auto-wrap I11 complet** : `return <appel externe/méthode>` dans une fn `Result`/`Option` → emballé `Ok`/`Some` **automatiquement** (plus de `ok(...)` manuel) |
| `10900` | **Générateurs `yield`** → itérateurs Rust (machine à états ou `gen`-block). **Dur** — à cadrer finement |
| `21300` | **Inférence élargie** : HOF utilisateur **fiable** (param-fonction appelé), trous résiduels comblés ; ambiguïtés numériques |

### D — Tooling & DX (tooling `7`)
| Spec | Contenu |
|---|---|
| `71000` | **`ironc fmt`** : formateur Iron canonique (indentation/espaces, ordre stable) — différé V7 (70600) |
| `71100` | **LSP** : serveur (diagnostics live, complétion, go-to-def, hover, rename) sur le pipeline existant. **Le grand levier DX** |
| `71200` | **`ironc repl`** : éval incrémental (compile-and-run par snippet) |

### E — Diagnostics & lint (diagnostics `6`)
| Spec | Contenu |
|---|---|
| `60500` | **Passe de lint réelle** (I4, différé V7/60300) : variables inutilisées, **clones évitables**, code mort, shadowing, aliasing — codes `IRN-Lxxx`, `--audit`/`deny` |
| `60600` | **Erreurs de type niveau Iron** (différé V7/60400) : détectées **avant `rustc`**, messages clairs, `Span` Iron |
| `60700` | **Source-maps granularité expression** (différé V6/60200) : marqueur invisible par expression, pas seulement par ligne |

### F — Validation finale (méta `9`)
| Spec | Contenu |
|---|---|
| `90400` | **Batterie « 30 vrais crates »** : projets réels texte/web/données/concurrence (extension de la batterie V7 avec de **vraies libs**, pas que de l'arithmétique) |
| `90100` | **Métrique de vérité CHIFFRÉE** (enfin, différée depuis V6) : correction `Iron→Rust` sur corpus complet + coût du clone-fallback (`--audit`) |
| `90500` | **Gate V9 final** : projet réel multi-fichiers (plusieurs crates + threading + stdlib) compile+s'exécute ; `fmt`/`lint`/`test` OK ; LSP répond ; métrique chiffrée ; **0 `unsafe`** |

## Décisions structurantes (à FIGER en phase 1)

- **✅ D0 — Pas de nouvel IR massif.** V9 est surtout du **comblement** sur l'IR existant (V7-D0/V8-D0). Les
  ajouts ponctuels (`Type::SharedSync`, `async`/`await` dans l'AST, `yield`) se gèlent en 1ʳᵉ vague.
- **✅ D1 — Protocoles bénis (51600).** Mapping **fixé** trait Rust → protocole Iron : `Iterator`→`for x in o`,
  `Display`/`ToString`→`str(o)`, `Add/Sub/Mul/Div`→`+ - * /`, `PartialEq/PartialOrd`→`== <`, `Index`→`o[i]`,
  `From/Into`→conversions, `Default`→constructeur. Les traits **user** se bindent par méthodes (UFCS) si
  monomorphisables, sinon `rust{}` (résidu 0.1%).
- **✅ D2 — `import` vs `use`.** `import regex` (haut niveau, auto-bind+cache, résout la version via
  `ironc.toml`) **désucre** vers le `use "….iri"` de V8 + injection de dépendance. `use` reste pour les
  `.iri` curés explicites. Rétro-compatible.
- **✅ D3 — `shared_sync` ≠ `shared`.** `shared` reste `Rc<RefCell>` (mono-thread, I5). `shared_sync` =
  `Arc<Mutex>` (multi-thread). Le passage à un `thread.spawn` **exige** `shared_sync` (le `rustc` filet I2
  rejette un `Rc` non-`Send`). Honnête : deux primitives, pas de magie.
- **🔶 D4 — Périmètre `async` (à arbitrer).** `async`/`await` mappés vers Rust async + **un runtime bindé**
  (tokio par défaut). Question de fond : runtime imposé ou choisi ? **Proposé** : tokio par défaut, surchargé
  via `ironc.toml`. Les patterns async exotiques (select!, pin manuel) → `rust{}` (0.1%).
- **🔶 D5 — `yield`/générateurs (à arbitrer).** Mapping vers un itérateur Rust (struct + état) ou `gen`-block
  nightly. **Proposé** : désucrage vers un itérateur à état pour les cas simples (`yield` en boucle) ; les
  coroutines complexes → hors-périmètre (0.1%). C'est le point le plus risqué de V9.
- **✅ D6 — Le 0.1% assumé.** Restent **hors V9, explicitement** : inférence Hindley-Milner **complète**,
  auto-hébergement (compilateur Iron en Iron), patterns async/coroutine exotiques, binding **100%-automatique**
  de **n'importe quel** crate (les traits user non-monomorphisables → `rust{}`). C'est le 0.1% honnête.

## Priorité / ordre des chantiers (vagues séquentielles)

1. **Fondation** (gel des ajouts IR : `shared_sync`, `async`/`await`, `yield`).
2. **A — Interop 100%** (trait-binding = le levier ; débloque la moitié « rust{} » → Iron pur).
3. **C — Polish cœur** (try/except, auto-wrap : fluidifie tout le reste).
4. **B — Concurrence** (shared_sync → thread → async).
5. **E — Diagnostics & lint** ∥ **D — Tooling** (sous-systèmes disjoints → parallèles).
6. **F — Validation** (batterie vrais crates → métrique chiffrée → gate V9).

## e2e / gate V9 (= « 99.9% fini »)

Un **projet Iron réel multi-fichiers** : importe ≥ 3 crates en `import` une-ligne (regex + serde_json +
un crate concurrent), utilise **threading** (`shared_sync` + `thread.spawn`) et des **méthodes de traits**
(itérateurs/Display) **en Iron pur**, compile en **Rust sûr (0 `unsafe`)**, s'exécute juste ; `ironc fmt`
formate, `ironc lint` émet des diagnostics justes, **LSP** répond (complétion/diagnostics), `ironc test`
passe ; la **métrique de vérité est chiffrée** sur le corpus complet + la batterie 30-vrais-crates. Remontée
I8 préservée partout.

## Hors V9 — le 0.1% honnête (→ jamais, ou recherche)

Inférence HM complète · auto-hébergement · async exotique (pin/select manuels) · coroutines complexes ·
binding **garanti** de tout crate (traits user non-monomorphisables → `rust{}`). Tout le reste **est fait**.
