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

## 🔲 Reste (greffes sur le socle, à corpus vert)
| Reste | Spec | Rayon de souffle |
|---|---|---|
| **`emit_use` unifié** : un seul point de consommation dans `lower` (args/récepteur/for/retours/champs/captures), subsume `byref_params`/for-élément/`extern_arg` | [40500] | élevé (toute l'émission) |
| **Modes de méthode** : `hit(&self, r: &Ray)` — exige `byref_params` dans les corps de méthode (absent) | [30800] 3.1 | élevé (toutes les méthodes du corpus) |
| **Last-use réel** : move/clone intra-expression (retire `.copy()`/`rec.clone()`) | [30800] 3.1 | moyen-élevé |
| **Désucrage opérateurs** : `a+b`→`a.__op__(b)` (parse) | [40500] 3.2 | moyen (garde primitifs) |
| **Gate raytracer** : clones résiduels éliminés + perf | 4.1 | — |

## Verdict honnête
Le **socle est posé et neutre** (la décision structurante du cadrage : le plan = vérité unique, prouvé sans
régression). C'est la marche **sûre** avant le rayon de souffle maximal. La suite (`emit_use` + exploitation)
est un **refactor cœur délicat** : il livre les wins visibles (raytracer sans clones résiduels, `a+b` sur
valeur empruntée), mais mérite une exécution **focalisée** avec le corpus-gate à chaque pas — pas une fin de
session précipitée qui risquerait l'invariant « corpus vert / diff maîtrisé ». Le 1ᵉʳ pilier (V16
interprocédural) + ce socle V17 (plan unique) sont en place ; l'exploitation se greffe dessus.
