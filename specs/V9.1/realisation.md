# V9.1 — Réalisation (tracker)

> **État : RÉALISATION COMPLÈTE — 23/23 pistes ✅ · GATE V9.1 VERT (21/21).** Toutes les specs V9.1
> planifiées **et** réalisées. (2.2 : granularité **ligne** livrée + utilisée par le debugger ; granularité
> **expression** explicitement reportée post-V9.1 — décision tranchée, plus un `🔶` ouvert.)

## Avancement des pistes (5 vagues, 23 pistes)

| Piste | Spec | Titre | Vague | Statut |
|---|---|---|---|---|
| 1.1 | 52000 | Principe « aucune lib Iron » + retrait io/os/time | 1 | ✅ fait |
| 1.2 | 52100 | `import <crate>` auto-bind complet (rustdoc + cache) | 1 | ✅ fait |
| 1.3 | 52200 | `import std` | 1 | ✅ fait |
| 1.4 | 52300 | Registre curé + `ironc add` | 1 | ✅ fait |
| 2.1 | 60800 | Diagnostics overlay rustc | 2 | ✅ fait |
| 2.2 | 60900 | Source-maps granularité expression | 2 | ✅ ligne (debug OK) · ⏸️ expr post-V9.1 |
| 2.3 | 71300 | LSP enrichi (complétion/hover/go-to-def) | 2 | ✅ fait |
| 2.4 | 71400 | `ironc fmt` (commentaires) | 2 | ✅ fait |
| 3.1 | 71500 | Extension VSCode : langage + icônes | 3 | ✅ fait |
| 3.2 | 71600 | Grammaire de coloration | 3 | ✅ fait |
| 3.3 | 71700 | Client LSP + linter VSCode | 3 | ✅ fait |
| 3.4 | 71800 | Gestionnaire d'imports VSCode | 3 | ✅ fait |
| 3.5 | 71900 | Build/run intégrés VSCode | 3 | ✅ fait |
| 4.1 | 21400 | Génériques utilisateur complets | 4 | ✅ fait |
| 4.2 | 21500 | Erreurs typées `Result[T, E]` | 4 | ✅ fait |
| 4.3 | 21600 | Closures avancées | 4 | ✅ fait |
| 4.4 | 72000 | `ironc repl` | 4 | ✅ fait |
| 4.5 | 72100 | Gestion de paquets | 4 | ✅ fait |
| 4.6 | 72200 | `ironc doc` | 4 | ✅ fait |
| 4.7 | 72300 | Intégration debugger | 4 | ✅ fait |
| 5.1 | 90600 | Le « 0.1 % » tracé | 5 | ✅ fait |
| 5.2 | 90700 | Batterie élargie 30 vrais crates | 5 | ✅ fait |
| **5.3** | 90800 | **Gate V9.1 final** | 5 | ✅ VERT (21/21) |

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
- **2.3** LSP enrichi : `completion` (mots-clés + types + symboles du fichier) · `hover` (signature en
  ```iron```) · `definition` (go-to-def intra-fichier). Tout dérivé de l'AST (front-end pur, pas de checker).
- **2.4** `ironc fmt` : normaliseur de lignes **idempotent** préservant les commentaires (le re-spacing
  token-à-token serait fragile en langage indentation-sensible). Recanonise l'indentation (4 espaces via
  pile), aligne les commentaires sur le code suivant, réduit les blanches, continuations `[...]` préservées.
- **Vague 3 COMPLÈTE** (extension VSCode `editors/vscode/`) : langage `.irn`/`.iri` + icône · coloration
  TextMate · client LSP + `ironc check` au save · `Iron: Add crate` · build/run + CodeLens « ▶ Run ».
- **4.1** génériques utilisateur (`def f[T]`/`class Pair[A,B]` + bornes ; borne `Clone` par défaut) → 7, 30.
- **4.3** closures avancées (type `fn(A)->R`, capture `move`, stockage/retour de closures).
- **4.4** `ironc repl` : compile-and-run par snippet (accumulateur + diff + rollback). A fait émerger un vrai
  trou de langage — **suites inline** (`def f(): return x`, `if c: y`) — désormais supportées dans `block()`.
- **4.5** paquets : `ironc add` (rustdoc→`.iri`) · `remove` · `update` (re-bind) — chaîne e2e prouvée (itoa).
- **4.6** `ironc doc` (overlay rustdoc → HTML).
- **4.2** erreurs typées `Result[T, E]` : variante additive `Type::ResultE(T,E)` (zéro régression sur les 21
  sites `Result(T)`). `Result[T]` reste `Result<T, IronError>`. `raise`/`err` → `Err(e.into())` (typé) ;
  `?` propage `E` ; `try/except e:` lie `e: E` (type capturé **du corps**, pas de la fn englobante) et le
  handler `match`e dessus. Mismatch → erreur rustc remontée (filet). Fixture `typed_errors.irn`.
- **4.7** debugger : `ironc build --debug` → binaire DWARF + `sourcemap.json` (ligne `.rs` → ligne `.irn`,
  I8) + `launch.json` CodeLLDB + commande VSCode `Iron: Build (debug)`. Flux prouvé bout-en-bout (gdb pose
  un breakpoint sur le `.rs`, la map retraduit l'arrêt en ligne Iron). Middleware DAP transparent = post-v0.
- **2.2** source-maps : la ligne marche déjà (granularité expression = raffinement non fait).
- **5.1** « 0.1 % » figé dans `generator.md` + messages d'orientation vérifiés (table ci-dessous).
- **5.2** batterie **30/30 VERT** (15 existants + 15 nouveaux crates auto-bindés via `ironc add` + `import`).
  **Iron-pur mesuré : 26/30 (87 %)** (4 `rust { }`, dans la fourchette V8 62–92 %). 16 via `import`, 11 via
  `use` manuel. Nouveaux : strsim · urlencoding · titlecase · is_prime · roman · deunicode · unidecode ·
  levenshtein · htmlescape · fastrand · pluralizer · gcd · distance · sift4 · primes.
- **5.3** GATE V9.1 final : `tests/gate-v9.1.sh` agrège **7 sections / 21 checks** — corpus · binding-30 ·
  métrique · outillage (check/lint/lib/fmt/repl/build--debug/doc/add) · LSP (init/completion/hover/def) ·
  **vitrine** (`tests/vitrine-v9.1/`, multi-fichiers, 0 unsafe) · artefacts VSCode. **VERT 21/21.**

**☐ RESTE** : — (V9.1 close ; raffinements optionnels = source-maps-expr, DAP transparent, couverture binder).

## Écarts / faiblesses (relevés)

- L'auto-bind produit des types (tuples/unit) que le parser `.iri` ne relisait pas → corrigé (cohérence
  bind↔parse à surveiller pour d'autres constructs).
- io/os/time **retirés** : un programme les utilisant doit migrer vers `import std`. Aucun test ne les
  utilisait (migration nulle).
- Le REPL étant ligne-à-ligne, les blocs multi-lignes ne s'y saisissent pas ; on a donc ajouté les **suites
  inline** au parser (`def f(): return x`). Limite assumée V0 : **une** instruction inline (pas de `a; b`
  chaîné). Couvre def/méthodes/if/while/for (tout passe par `block()`).
- 4.2 a révélé un bug **préexistant** : un `try/except` au *tail* d'une fn renvoyant une valeur ne compilait
  pas (E0317, chute non typée vue par rustc) — y compris en `IronError`. Corrigé généralement : quand le
  corps du `try` diverge sur le chemin de succès (`definitely_returns`), le `if let` reçoit `else
  unreachable!()` ⇒ totalité prouvée. Bénéficie aussi au cas `IronError`.
- Extern `Result[T, E]` (binding `.iri`) hors-scope V0 : les fns externes restent `Result[T]`→`IronError`
  (le mapping `.map_err(IronError)` ne s'applique qu'à `Result(_)`). Erreurs typées = fns Iron pour l'instant.
- **5.2 a révélé deux vrais bugs de langage** (le but même de la batterie) — corrigés :
  - **Littéraux non-ASCII mojibakés** : le lexer poussait `octet as char` (Latin-1) → `"café"` devenait
    `Café`. Corrigé : `push_char` décode le code point UTF-8 complet (`lex_string`/`lex_fstring`/`lex_char`).
  - **`len(str)` rendait des octets** (`.len()`) au lieu de caractères (sémantique Python) → `len("naïve")`
    valait 6. Corrigé : `len`/`.len()` type-aware → `chars().count()` pour `str`, `.len()` pour list/dict/set.
  Fixture `utf8_strings.irn` gèle les deux.
- `ironc add` normalise désormais le **nom de fichier** `.iri` (tirets → underscores) : `import num_integer`
  trouve `num_integer.iri` (le nom de crate garde son tiret pour cargo). Débloque les crates hyphénés.
- Couverture du binder **mesurée honnêtement** : ~30 crates sondés, ~15 bindés proprement « Iron-pur » du
  premier coup ; les ratés typiques = génériques `AsRef`/`impl Trait`, retours `Cow`, traits non bénis,
  params `&[u8]`. Conforme au « 0.1 % » (binding non garanti → `.iri` curé ou `rust { }`).

## Hors V9.1 (le « 0.1 % », spec 90600) — **tracé + messages d'orientation vérifiés**

Liste figée dans `generator.md` (§1, encadré sous I14). Chaque catégorie a une **issue orientée** (critère
d'acceptation 2), vérifiée sur ≥ 1 exemple :

| Catégorie | Issue / message vérifié |
|---|---|
| HM globale | annotation requise → `rustc` réclame le type, **remonté en Span Iron** (overlay 60800). |
| Auto-hébergement | N/A (contrat doc : `ironc` reste en Rust). |
| Async exotique (`select!`…) | `rust { }` (échappatoire universelle — prouvée : showcase `sha2::Digest`). |
| Coroutines complexes | `yield from` → **message orienté** « utilise `for…: yield x` ou `rust { }` » (parser). |
| Binding non garanti | `.iri` note « N non mappable(s) sauté(s) » ; fournir `.iri` curé ou `rust { }`. |

Tout le **reste** est dans une spec V9.1 ci-dessus, **toutes planifiées + réalisées**.
