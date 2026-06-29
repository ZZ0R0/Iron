# V17 — ÉTAT RÉEL D'IMPLÉMENTATION (phase 3)

> **Honnête.** Le **socle est amorcé et prouvé neutre** : l'`OwnershipPlan` est la sortie unique du moteur,
> diff Rust **vide** (corpus+raytracer). L'**unification `emit_use` complète** et l'**exploitation** (modes de
> méthode, last-use, opérateurs) restent — à fort rayon de souffle, greffées sur ce socle, corpus vert à chaque pas.

## ✅ Fait et vérifié

### Vague 1.1 — schéma `OwnershipPlan` (ir.rs)
`struct OwnershipPlan { sites: OwnershipMap, params: ParamModes }` — la **seule** sortie ownership, subsume les
deux maps.

### Vague 2.1 (partie) — le plan = sortie unique, NEUTRE
- `resolve(ast, types) -> OwnershipPlan` (mêmes valeurs, regroupées) ; `driver` consomme le plan.
- **NEUTRALITÉ PROUVÉE** : `ironc emit` raytracer avant/après → **diff VIDE** (Rust généré identique). 4 suites
  tests + corpus échantillon verts. → Le plumbing passe par le plan **sans régression**.

### Vague 3.1 (partie) — **modes de méthode** (exploitation) — **WIN MAJEUR**
- `resolve.rs` : analyse **uniquement** les méthodes **inhérentes** (trait_name vide) → les méthodes de trait
  restent by-value (matchent la signature du trait). `lower.rs` : `lower_method(type_name, m, use_modes)` émet
  `&T` pour les params ByRef (inhérentes) ; le site d'appel résout la méthode via `TypeMap` → modes `Type::m`
  → `&arg` (double-`&` absorbé par la déréf-coercition Rust). `use_modes=false` pour trait/dunder (sûr).
- **Effet (raytracer)** : `fn hit(&self, r: &Ray, …)` ; `obj.hit(&r)` sans `.clone()` ; et **surtout**
  `Vec3.add/sub/dot/scale(&Vec3)` — plus de clone de `Vec3` par appel (**des millions** sur le hot path).
- **MESURE** : **0.102 s** vs 0.301 s (V16) vs **0.51 s baseline** → **~5× plus rapide que baseline**. Le Rust
  émis **compile + run + image valide**. Corpus large vert (semver/nalgebra/num-rational/rust_decimal/arrayvec/roaring).
- **Résiduel** : **7 clones** dans tout le raytracer, **majoritairement légitimes** (construction de données
  possédées : `HitRecord` possède son `Material`, `Ray` possède origin/direction). Seul `nearest = rec.clone()`
  est un candidat **last-use** (HitRecord mineur).

## 🔲 Reste (greffes sur le socle, à corpus vert)
| Reste | Spec | Rayon de souffle |
|---|---|---|
| **Last-use réel** : move/clone intra-expression (retire le `rec.clone()` résiduel / le `.copy()` manuel) | [30800] 3.1 | moyen-élevé |
| **Désucrage opérateurs** : `a+b`→`a.__op__(b)` (parse) | [40500] 3.2 | moyen (garde primitifs) |
| **`emit_use` pleinement unifié** : finir de router tous les sites par un seul point (le site d'appel l'est déjà pour fns+méthodes) | [40500] | moyen |
| **Gate raytracer final** : clones résiduels (mineurs/légitimes) + perf consignée | 4.1 | — |

## Verdict honnête
Le **socle est posé et neutre** (la décision structurante du cadrage : le plan = vérité unique, prouvé sans
régression). C'est la marche **sûre** avant le rayon de souffle maximal. La suite (`emit_use` + exploitation)
est un **refactor cœur délicat** : il livre les wins visibles (raytracer sans clones résiduels, `a+b` sur
valeur empruntée), mais mérite une exécution **focalisée** avec le corpus-gate à chaque pas — pas une fin de
session précipitée qui risquerait l'invariant « corpus vert / diff maîtrisé ». Le 1ᵉʳ pilier (V16
interprocédural) + ce socle V17 (plan unique) sont en place ; l'exploitation se greffe dessus.
