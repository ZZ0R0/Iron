# Iron — `specs/generated.md`

> **Découpage en versions**, produit à partir de `generator.md` (roadmap §8 + invariants I1–I14).
> Pipeline : `cadrage → generator.md (règles) →` **`generated.md (découpage en N versions)`** `→ specs/Vn/generated.md (carte d'une version) → specs/Vn/XXXXX.md (specs) → planification/X.Y.md → implémentation`.
> `generator.md` possède le **contrat de contenu** (gabarits, invariants, nommage `XXXXX`). Ce fichier **répartit** seulement la roadmap en versions ; il **ne crée aucun invariant**.
> **Numérotation** : on démarre à **V0** (le dossier existe ; label V0/V1 indifférent — décision utilisateur). `generator §3` appelle cette 1ʳᵉ version « V1 » et `§8` en donne une carte *illustrative* ; ce fichier la **réalise** et la supersède. (→ aligner `generator §3` si souhaité, non touché ici.)
> Statut global : **🔶 proposé**.

## Principe de découpage

- **Planifier les N versions d'un coup** (visibilité), **exécuter version par version** (un cycle complet des 4 phases par `Vn`, on teste, les corrections alimentent la suivante — cf. `CLAUDE.md`).
- **V0 = le plus petit système *entièrement fonctionnel*** (la boucle source Iron → Rust → `cargo` vert → exécution correcte, **source maps comprises**), pas le plus petit système. Le difficile est **repoussé** en fin de plan.
- **Dérisquage** : les **source maps** (risque n°1, `generator §8`) sont **dans V0** — I8 les exige dès le codegen, on ne les reporte pas.

## Carte des versions

| Version | Cœur | Specs (`generator §8`) | Milestones | Gate e2e (phase 4) | Statut |
|---|---|---|---|---|---|
| **V0** | Boucle source→binaire qui compile + source maps | 10100, 10200, 20100, 20200, 30100, 40100, 60100, 70100 | M0–M3 | `ironc build/run` sur un programme représentatif → Rust → `cargo` vert → exécution correcte ; spans `.irn`↔Rust ; zéro `unsafe` (I2) | 🔶 **détaillée** |
| **V1** | Qualité du Rust : élision d'emprunt | 30200 | M2+ | clone-fallback → emprunt là où prouvé sûr ; `--audit` liste les fallbacks ; **sémantique inchangée** vs V0 ; aliasing mutable jamais auto-cloné (I3) | 🔶 provisoire |
| **V2** | Interop Rust par niveaux (I7) | 50100, 50200, 50300 | M4 | appel de crates réelles aux 3 niveaux ; ordre I7 respecté (50300 en fin) | 🔶 provisoire |
| **V3** | Système de types avancé | 20300 | M5 | enums + match + traits + génériques → Rust idiomatique ; projection I6 tient | 🔶 provisoire |
| **V4** | Stdlib | 80100 | M6 | surface stdlib utilisable (wrappers fins, pas de runtime lourd I5) ; I7 | ✅ réalisé |
| **V5** | Complétude : `move` (I1), `inout` (I3), inférence méthode, NaN (I12), métrique | G1–G8 | M2++ | invariants comblés ; corpus inchangé ; **0 clone** mesuré | ✅ réalisé |
| **V6** | Finition : les **18 manques** de l'audit (I11 flux, I5 `Rc`, closures, modules…) | 20400 30300 20700 20500 20600 30400 50400 70200 80200 10300 (60200·90100 différés) | M5+/M6+ | I11/I5 honorés + dettes payées | ✅ **réalisé** (10/12) |
| **V7** | **« Langage réel »** : langage-cœur (tuples/f-strings/compréhensions/numériques), crates Rust réelles, stdlib solide (wrappers `std`), lint, tooling/cargo | A:10400–21000 · B:50500–50800 · C:80300–80800 · D:60200–60400 · E:70300–70600 · F:30500–90200 (~28) | M1–M7 | Iron **utilisable pour de vrai** | ✅ **réalisé** (batterie 30/30) |
| **V8** | **« Écosystème »** : **auto-binding des crates Rust** — types natifs récursifs, POO (`impl`/méthodes/ctors), génériques (instance-pinning), **reconnaisseurs d'idiomes** (builder/getter/iter/closure) | 50900 51000 51100 51200 51300 51400 51500 · 21100 · 40200 · 90300 (10) | M8 | `ironc bind regex/serde_json/csv` → API Iron **style Python** ; couverture 62–92 % ; 0 `unsafe` | ✅ **réalisé** (10 crates réels, gate VERT) |
| **V9** | **« Clôture — fini à 99.9% »** : interop 100% (trait-binding, `import`, registre, inverse Iron→Rust, **attributs pass-through**), **concurrence** (threading + async), polish cœur (try/except, I11, yield, inférence), **tooling** (fmt, LSP, REPL), diagnostics & lint réels, **métrique chiffrée** | A:51600–70800·51900 · B:30700·70900·21200 · C:40300·40400·10900·21300 · D:71000–71200 · E:60500–60700 · F:90400·90100·90500 (22) | M9 | projet réel multi-crates + threading + tooling complet ; **métrique de vérité chiffrée** ; 0 `unsafe` | 🟡 **réalisé ~74% (17/23, dont async/yield/LSP)** |
| **V9.1** | **« Langage pour devs »** : **zéro lib Iron** (tout par import Rust + `import std`), support langage via **overlay rustc** (diagnostics/LSP), **intégration VSCode** (fichiers/coloration/LSP/imports/build), profondeur (génériques, erreurs typées, closures), maturité (paquets, doc, debug) | A:52000–52300 · B:60800·60900·71300·71400 · C:71500–71900 · D:21400·21500·21600·72000–72300 · E:90600·90700·90800 (23) | M9.1 | un dev code dans VSCode, n'importe que des crates Rust, diagnostics/complétion/build live, `0 unsafe` | ✅ **réalisé — GATE V9.1 VERT** (23/23 pistes · 21/21 checks · vitrine 0 unsafe) |

> **V0–V8 réalisés**, chacun avec sa trace `specs/Vn/realisation.md`. **V7** : clone Python (batterie 30/30).
> **V8 réalisé** (`specs/V8/`) : auto-binding prouvé sur **10 vrais crates** (regex, serde_json, csv, ureq/HTTP,
> std::thread, sha2, chrono…), gate binding VERT, couverture auto 62–92 %.
> **V9 cadrée + planifiée** (`specs/V9/`) : la **version de clôture** — 6 chantiers, 21 specs, 22 pistes en
> 6 vagues. Ferme **tout** ce qui restait (le résidu `rust{}` via trait-binding, threading/async natifs,
> tooling fmt/LSP/REPL, lint/diagnostics réels, `import` ergonomique + registre, inverse Iron→Rust, métrique
> chiffrée). Décisions D0–D6 (🔶 D4 async-runtime, D5 yield à arbitrer). **À la fin de V9 : langage fini à
> 99.9%** — reste le 0.1% assumé (HM complet, auto-hébergement, async exotique, binding garanti de tout crate).
> **V6 réalisé** : 10/12 specs implémentées + prouvées (gate **vert** : 37/37 corpus, 0 `unsafe`, 3 clones
> intentionnels, remontée I8 OK, modules multi-fichiers OK). **2 différées par conception** : 60200
> (source-maps expression — mécanisme 🔶-ouvert) et 90100 (métrique de vérité — exige le gel du langage).
> Détail & faiblesses→V7 : `specs/V6/realisation.md`.

## Pourquoi cet ordre

- **V0** se justifie seule : c'est le **squelette e2e** de la méthodo (phase 3) élevé en version testable. Tout le reste s'y greffe. Source maps dedans (dérisquage n°1).
- **V1** (élision d'emprunt) est **perf, pas correction** : V0 est déjà correct par clone-fallback (I1). On stabilise le cœur avant d'optimiser → repoussé.
- **V2** (interop) **avant** les types avancés : le bloc `rust:` inline est une **échappatoire** — disposer de l'interop permet de contourner ce qui manque encore côté langage.
- **V3** (génériques/traits) = le **morceau dur** conceptuellement → isolé tard pour ne pas bloquer le cœur.
- **V4** (stdlib) dépend de l'interop (V2) et des types (V3) → **dernier**.

## Exécution

- Un cycle 4-phases par version ; on ne mélange pas deux versions. Fin de `Vn` → phase 4 (tests + notes de faiblesses/dettes) → cadrage de `Vn+1`.
- La transition `Vn → Vn+1` est une **décision de l'utilisateur**.
