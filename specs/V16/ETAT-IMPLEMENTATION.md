# V16 — ÉTAT RÉEL D'IMPLÉMENTATION (phase 3/4)

> **Honnête, mesuré.** La **priorité #1-2 du rapport** (ownership interprocédural → scène par `&` ; for-élément
> emprunté → objets non clonés) est **livrée et vérifiée e2e** sur le ray tracer. Le reste ([30500] modes de
> méthode + last-use, [50700] imports, [21800] inférence, [30500§2.1] opérateurs) est cadré/planifié mais **non
> réalisé** — checkpoint pris car le gain dominant est acquis et la suite a un rayon de souffle élevé pour des
> gains plus petits.

## ✅ Livré et vérifié (run réel)

### Vague 1.1 — escape analysis interprocédurale à point fixe (fns libres) [30500]
- `resolve.rs` : remplacé l'analyse intra-procédurale + whitelist `print`/`len` par un **point fixe**
  interprocédural des modes de params. Init optimiste `ByRef` ; rétrogradation **monotone** `ByValue`
  (terminaison) ; un argument à une position **`ByRef` du callee** n'échappe pas (modes courants consultés,
  méthodes résolues via `TypeMap`) ; **sur-approximation conservatrice** (doute → `ByValue`).
- **Effet (raytracer)** : `ray_color(scene: &Scene, r: &Ray, …)`, `shade(mat, scene: &Scene, r: &Ray,
  rec: &HitRecord, …)`, `reflect`/`refract`/`default_sky` en `&`. **La scène n'est plus clonée** par échantillon
  ni par rebond — *la racine du `RAPPORT_IRON.md`*.

### Vague 2.1 (partie) — élément de boucle emprunté [30500 §1 corollaire]
- `lower.rs` : `for_var_borrowable()` (analyse conservatrice single-var) → `for x in xs.iter()` (élément `&T`)
  au lieu de `xs.iter().cloned()` quand le corps est read-only.
- **Effet** : `for obj in self.objects.iter()` (×2, `hit`/`occluded`) — les objets ne sont plus clonés à chaque
  test de rayon (2ᵉ clone scene-size).

### Vérification e2e (gate partiel)
- **Le Rust émis du raytracer COMPILE** (`&Scene`/`&Ray`/`&élément` valides), **tourne**, produit un
  **PNG 1920×1080 correct** (~identique à la référence).
- **Perf mesurée** : **0.301 s** (V16 1.1+2.1) vs **~0.51 s** baseline V15 (scène+objets clonés) ≈ **1.7×**.
  Conforme à la prédiction du rapport (« ~1,5× croissant avec le nombre d'objets »).
- **Corpus** : échantillon vert (hello, nalgebra) + **4 suites de tests unitaires vertes** ; 0 `unsafe`.

## 🔲 Cadré/planifié mais NON réalisé (checkpoint)

| Reste | Pourquoi pas fait ici | Gain |
|---|---|---|
| **Modes de méthode** ([30500]) | `method_body_block` ne pose pas `byref_params` aujourd'hui → machinerie **nouvelle** (byref-in-methods) sur **toutes** les méthodes du corpus = rayon de souffle élevé | retire `r.clone()` par objet (Ray 48 o) |
| **Last-use réel** ([30500] §2.2) | analyse de dernière-utilisation intra-expression (remplace single-use) | retire `rec.clone()` / le `.copy()` manuel |
| **Import `.iri` récursif** ([50700]) | piste indépendante (driver.rs) non démarrée | ergonomie modules |
| **Inférence whole-program** ([21800]) | piste indépendante (infer/) non démarrée | retire des annotations forcées |
| **Désucrage opérateurs** ([30500 §2.1]) | parse/lower | `a+b` sur valeurs empruntées |

## Verdict honnête
**Le « problème de fond » du rapport (clonage par défaut, plafond de perf) est CORRIGÉ pour les fonctions** :
la scène et les objets — les deux clones **proportionnels à `objects.len()`** que le rapport jugeait
« rédhibitoires » — ne sont **plus clonés**. Gain **1.7× mesuré**, e2e vert. Les clones résiduels (`r`, `rec`)
sont **constants/petits** et demandent la machinerie byref-in-methods + last-use (à fort rayon de souffle), à
faire dans une passe dédiée avec le corpus comme garde. Le squelette (point fixe interprocédural) **est posé**.
