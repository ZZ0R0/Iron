# Iron — V7 / cadrage (`README.md`)

> **Version « langage réel »** : on fait passer Iron de « transpileur qui marche » à **langage qu'on
> utilise pour de vrai** — langage-cœur complet, crates Rust réelles, stdlib solide, lint, tooling/cargo.
> Parent : `specs/generated.md` (V7 ajoutée). Contrat de contenu : `../generator.md`.
> Statut : **🔶 cadrage** (architecture + décisions structurantes posées ; specs détaillées, planif et
> réalisation à suivre, sur feu vert et après arbitrage des `🔶` ci-dessous).
>
> **La plus grosse version du projet** (~28 specs, > V0→V6 réunis). Exécutée en **6 chantiers** séquencés,
> chacun = fondation → fan-out → unification → checkpoint. `async`/lifetimes/LSP/auto-hébergement = **V8+**.

## Carte des specs (6 chantiers, ~28 specs)

### A — Langage cœur (frontend `1`, types `2`)
| Spec | Contenu |
|---|---|
| `10400` | **Tuples** : type `(A, B)`, littéral, destructuring `a, b = …`, **retour multiple** |
| `10500` | **f-strings** / interpolation `f"{x} = {y}"` → `format!` |
| `10600` | **Compréhensions** liste/dict/set (`[f(x) for x in xs if p(x)]`) |
| `10700` | **Affectation augmentée** (`+=`,`-=`,…) + opérateurs `**`, `//`, bit-à-bit |
| `10800` | **Ternaire** `a if c else b`, `in`/`not in`, `assert`, **slicing** `xs[1:3]` |
| `20800` | **Types numériques élargis** : `u64`/`usize`/`i32`/`u8`/`f32`/`char` + overflow (cf. 🔶-D1) |
| `20900` | **Type `set`** : littéral, opérations (union/inter/diff/membership) |
| `21000` | **Variantes à champs nommés** (différé V6/20600) : `enum E: V(a: i64, b: str)` |

### B — Crates Rust réelles (interop `5`)
| Spec | Contenu |
|---|---|
| `50500` | **`bind` complet** : mapper `str`/`String`/collections/structs publics ; **chemins sous-module** |
| `50600` | **Dépendances de crates** : déclaration (`ironc.toml`) + résolution + auto-`bind` (cf. 🔶-D5) |
| `50700` | Appel de **fns Rust génériques** + **impl de traits std** (`Display`/`Iterator`/`From`) |
| `50800` | Forme indentée **`rust:`** (différé V6) |

### C — Stdlib solide (stdlib `8`)
| Spec | Contenu |
|---|---|
| `80300` | **Fichiers IO** : `read_file`/`write_file`/`read_lines` |
| `80400` | **JSON** : `parse`/`to_json` (via `serde_json` bindé) |
| `80500` | **Collections** : `set`/`deque`/ordered ; `dict.keys/values/items`, méthodes de `list` |
| `80600` | **Combinateurs `Option`/`Result`** : `map`/`and_then`/`unwrap_or`/`is_some`/… |
| `80700` | **Itertools** : `enumerate`/`zip`/`reversed`/`reduce`/`sorted(key=)`/`any`/`all` |
| `80800` | **datetime** / **random** / **regex** / **os·env·process** (via crates bindées) |

### D — Lint & diagnostics (diagnostics `6`)
| Spec | Contenu |
|---|---|
| `60300` | **Passe de lint réelle** (I4) : variables inutilisées, clones évitables, shadowing, code mort, aliasing |
| `60400` | **Erreurs de type niveau Iron** (avant `rustc`) : messages clairs, `Span` Iron |
| `60200` | **Source-maps granularité expression** (différé V6 ; mécanisme « marqueur invisible ») |

### E — Tooling & cargo (tooling `7`)
| Spec | Contenu |
|---|---|
| `70300` | **Manifeste `ironc.toml`** : `[package]`/`[dependencies]`/`[iron]` → `Cargo.toml` (cf. 🔶-D4) |
| `70400` | **`ironc new`** : scaffolding d'un projet (manifeste + `src/main.irn`) |
| `70500` | **`ironc test`** : runner (`test fn` / `assert`) → `cargo test` |
| `70600` | **`ironc fmt`** : formateur Iron (indentation/espaces canoniques) |

### F — Finition ownership & méta (ownership `3`, meta `9`)
| Spec | Contenu |
|---|---|
| `30500` | **Escape point-fixe** (DV2, inter-signatures) + **scoping for-var** post-boucle (D7b) |
| `30600` | **`weak()`/`Weak`** + lint aliasing I3 (finition Rc, différé V6) |
| `90200` | **Durcissement du corpus** vers la métrique de vérité `90100` (chiffrée quand le langage gèle) |

## Décisions structurantes (à FIGER en phase 1 — `🔶` = proposé, j'attends ton arbitrage)

> Tout ce qui est **partagé entre composants** se fige ici, sinon deux agents le comblent différemment
> en phase 3 (cf. CLAUDE.md). Mes choix de **design** = `✅` ; les choix qui touchent au **fond / au feel
> du langage** = `🔶` proposé, à valider.

- **✅ D0 — Fondation IR (1ʳᵉ vague, gelée).** Nouveaux contrats `ir.rs` : `Type::Tuple(Vec<Type>)`,
  `Type::Set(Box<Type>)`, `Type::Num(NumKind)` (élargit `I64`/`F64`), `ExprKind::{Tuple, FString, Comp,
  Ternary, Slice}`, `Pattern::Tuple`, `StmtKind::Assert`, variantes enum à champs nommés. Gelés avant tout
  fan-out, comme la fondation V6.
- **✅ D1 — Types numériques : zoo Rust complet** *(tranché)*. Défauts inchangés (`i64`/`f64`, littéraux non
  suffixés) ; **tout** le reste accessible **par annotation** : `i8`…`i128`, `u8`…`u128`, `usize`/`isize`,
  `f32`, `char`. **Overflow** = sémantique `rustc` (panic debug / wrap release) — filet I2. Coût : border
  l'**ambiguïté numérique** à l'inférence (le défaut `i64`/`f64` reste, annotation = type concret).
- **✅ D2 — Stdlib = modules namespacés, IMPLÉMENTÉS en wrappers minces sur `std` + crates** *(tranché,
  affiné par l'utilisateur)*. On **ne réinvente pas** une stdlib Iron : on **habille la stdlib Rust** comme
  le reste de l'interop. `io.read_file` → `std::fs::read_to_string`, `json.parse` → `serde_json`, `os.env`
  → `std::env`. Cœur plat conservé (`print`/`len`/`range`/`str`/…). **Conséquence : le chantier C fusionne
  avec B** — la stdlib = des **modules-wrappers « bénis »** (`.iri` curés + namespacing). Respecte I5 (pas
  de runtime), I6/I7 (projection propre / interop par niveaux). Moins de code bespoke.
- **✅ D3 — `set`/`dict` ⇒ `HashSet`/`HashMap`** (déjà le cas pour dict) ; clés `Hash+Eq` (rustc filet).
  `deque` ⇒ `VecDeque`, ordered ⇒ `BTreeMap`.
- **✅ D4 — Schéma `ironc.toml`** : `[package] name/version/edition`, `[dependencies] crate = "x.y"`,
  `[iron] entry = "src/main.irn"`. Projeté 1-1 vers `Cargo.toml`. C'est le **contrat** tooling↔deps↔bind.
- **✅ D5 — Crates : hybride, pas 100 % automatique** *(tranché)*. `bind` via nightly `rustdoc-json` pour
  le cas courant (best-effort, **dégradation propre** : items non mappables sautés + comptés). Pour les
  crates clés (serde, regex…) où le format rustdoc-json est instable, on **autorise un `.iri` curé**
  fourni. Iron **ne garantit pas** le binding auto de n'importe quelle crate — limite assumée. Les
  modules-stdlib (D2) sont précisément ces `.iri` curés « bénis ».
- **✅ D6 — Format de lint** : chaque lint porte `Span` Iron + sévérité (`warn`/`deny`) + code (`IRN-Lxxx`),
  émis par `--audit` (étend l'existant) ; jamais bloquant sauf `deny` explicite (I4).
- **✅ D7 — Désucrage** : f-strings → `format!`, compréhensions → chaînes d'itérateurs, `+=` → `x = x + …`,
  ternaire → `if/else` expression, slicing → `[a..b].to_vec()`. Aucun runtime nouveau (I5).

## Priorité / ordre des chantiers (vagues séquentielles)

1. **Fondation D0** (gel IR) — 1ʳᵉ vague, comme V6.
2. **A — langage cœur** (le plus visible ; débloque le reste : f-strings/tuples utilisés partout).
3. **B — crates réelles** + **D4/D5** (déblocage interop ; conditionne la stdlib C).
4. **C — stdlib** (s'appuie sur B + D2).
5. **D — lint & diag** + **E — tooling/cargo** (parallélisables : sous-systèmes disjoints).
6. **F — finition ownership + méta** ; gate V7.

## e2e / gate V7

Au-delà du corpus : un **projet réel multi-fichiers** (`ironc.toml` + crates + stdlib) utilisant tuples,
f-strings, compréhensions → compile + s'exécute ; `ironc new/test/fmt` fonctionnent ; **lint** émet des
diagnostics justes ; **0 `unsafe`**, clones maîtrisés. Corpus durci. Remontée I8 préservée.

## Hors V7 (→ V8+, honnête)

`async`/`await`, générateurs (`yield`), **lifetimes / retour d'emprunts**, inférence Hindley-Milner
complète, **LSP**, REPL, auto-hébergement, métrique de vérité **chiffrée** (90100, attend le gel).

## Note

Specs au gabarit `generator §4`, à détailler **après arbitrage des `🔶`** (D1, D2, D5 surtout) — ils
conditionnent plusieurs specs. Planification (vagues/pistes) ensuite.
