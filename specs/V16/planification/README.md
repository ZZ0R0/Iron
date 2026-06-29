# V16 — PLANIFICATION — Ownership whole-program (réalise [30500] · [50700] · [21800])

> Découpage **d'exécution** (vague.piste), pas de cadrage. Chaque unité **cite** la/les spec(s) V16 qu'elle
> réalise. **Invariant central de toute la phase 3 : le corpus 80 reste VERT à chaque étape** (l'e2e de V16),
> et la **boucle d'itération raytracer** valide le « bon Rust généré » (`&Scene`/`&Ray`/`&élément`, sans clone).

## Principe : squelette d'abord, fan-out où c'est vraiment disjoint

Le cœur ([30500]) est un **refactor couplé** de `resolve.rs` (l'analyse) + sa consommation dans `lower.rs` —
donc **séquentiel**. Les frictions secondaires ([50700] driver.rs, [21800] infer/) sont **file-disjointes** →
vraies pistes **parallèles**. Le désucrage d'opérateurs partage `lower.rs` avec l'analyse → séquencé après.

## Carte des vagues

| Vague | Piste | Réalise | Fichiers POSSÉDÉS | Parallèle ? |
|:--:|:--:|---|---|---|
| **1** | 1.1 | [30500] | `resolve.rs` (call graph + point fixe modes fns+méthodes), `lower.rs` (consommation modes méthodes) | séquentiel — **squelette** |
| **2** | 2.1 | [30500] | `resolve.rs` (for-élément + last-use réel), `lower.rs` (émission for-élément) | séquentiel après 1.1 |
| **2** | 2.2 | [50700] | `driver.rs` (résolution import récursive) | **‖** (disjoint) |
| **2** | 2.3 | [21800] | `infer/` (propagation inter-appels + membre non-appelé + annotation de retour) | **‖** (disjoint) |
| **3** | 3.1 | [30500] §2.1 | `parse.rs` (désucrage `a OP b`→`a.__op__(b)`), `lower.rs` (retrait codegen opérateur) | séquentiel (partage `lower.rs`) |
| **4** | 4.1 | toutes | unification : corpus 80 + raytracer complet (emit→build→run) + perf + polish §2.7 | séquentiel — **gate final** |

## Graphe de dépendances

```
1.1 ─┬─► 2.1 ─► 3.1 ─┐
     │                ├─► 4.1 (unification + gate)
     ├─► 2.2 ─────────┤
     └─► 2.3 ─────────┘
```
- **1.1** = squelette : fige le **call graph** + le **contrat de modes whole-program**. Sans lui, rien.
- **2.1** (resolve/lower) séquentiel après 1.1 (même fichiers). **2.2** (driver) **‖ 2.3** (infer) — disjoints,
  démarrables dès 1.1 figé (2.3 construit son propre call graph ou réutilise celui de 1.1 si exposé proprement).
- **3.1** après 2.1 (partage `lower.rs`).
- **4.1** = unification finale + gate e2e.

## L'invariant e2e & la boucle d'itération (le critère de « fini »)

À **chaque** piste, deux vérifications :
1. **Corpus 80 reste VERT** (zéro régression compile/run) — re-mesuré.
2. **Raytracer** (boucle rapide via `ironc emit`, sans compiler) : les signatures attendues apparaissent
   progressivement dans le Rust généré :
   - après 1.1 : `fn ray_color(scene: &Scene, r: &Ray, …)`, `fn shade(mat: &Material, scene: &Scene, …)`,
     `fn hit(&self, r: &Ray, …)` ;
   - après 2.1 : `for obj in &self.objects` (plus de `.iter().cloned()`) ; plus de `.clone()` parasites ; le
     `.copy()` manuel devient inutile.
   - **gate 4.1** : `ironc emit` raytracer → `cargo build --release` vert → `./iron_out` produit `output.png`
     **identique** à l'oracle, et **plus rapide** que la version V15 (gain mesuré).

> **Contrat de sûreté (rappel du cadrage)** : optimiste `ByRef` → rétrogradation **monotone** (terminaison) ;
> **dans le doute → `ByValue`** (clone, toujours correct). Un `&T` qui casserait `rustc` = bug bloquant.

## Test mental de non-collision (validé)

- 2.1 `resolve.rs`/`lower.rs` · 2.2 `driver.rs` · 2.3 `infer/` → **fichiers disjoints**, parallélisables.
- 3.1 partage `lower.rs` avec 2.1 → **séquencé** (pas parallèle avec 2.1). ✅

> **Méthode.** Planif figée ici. **Prochaine étape = réalisation sur ton feu vert** (autonomie de phase 3 :
> squelette → fan-out → unification, sans m'arrêter entre vagues ; seul gate = 4.1 ou vrai blocage).
