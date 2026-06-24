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
| **V4** | Stdlib | 80100 | M6 | surface stdlib utilisable (wrappers fins, pas de runtime lourd I5) ; I7 | 🔶 provisoire |

> V0 est détaillée dans `specs/V0/generated.md`. V1–V4 ont une carte **provisoire** (`specs/Vn/generated.md`) : détail complet produit **au début de leur cycle**, corrections de Vn-1 intégrées.

## Pourquoi cet ordre

- **V0** se justifie seule : c'est le **squelette e2e** de la méthodo (phase 3) élevé en version testable. Tout le reste s'y greffe. Source maps dedans (dérisquage n°1).
- **V1** (élision d'emprunt) est **perf, pas correction** : V0 est déjà correct par clone-fallback (I1). On stabilise le cœur avant d'optimiser → repoussé.
- **V2** (interop) **avant** les types avancés : le bloc `rust:` inline est une **échappatoire** — disposer de l'interop permet de contourner ce qui manque encore côté langage.
- **V3** (génériques/traits) = le **morceau dur** conceptuellement → isolé tard pour ne pas bloquer le cœur.
- **V4** (stdlib) dépend de l'interop (V2) et des types (V3) → **dernier**.

## Exécution

- Un cycle 4-phases par version ; on ne mélange pas deux versions. Fin de `Vn` → phase 4 (tests + notes de faiblesses/dettes) → cadrage de `Vn+1`.
- La transition `Vn → Vn+1` est une **décision de l'utilisateur**.
