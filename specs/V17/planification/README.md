# V17 — PLANIFICATION — Moteur d'ownership unifié (réalise [30800] · [40500])

> Découpage d'**exécution** (vague.piste), pas de cadrage. Chaque unité **cite** la/les spec(s) V17. **La
> stratégie est dans le découpage** : on pose le socle en **étape NEUTRE** (le moteur reproduit l'existant →
> **diff Rust vide** sur corpus+raytracer), PUIS on **exploite** (greffe site-par-site, chacun retirant une
> règle ad-hoc), **corpus vert à chaque pas**.

## Principe : neutre d'abord, exploitation ensuite

Le socle ([30800]+[40500]) est un **refactor couplé** `resolve.rs`↔`lower.rs` — séquentiel. **Mais** la
neutralité le rend **sûr** : on prouve que le nouveau moteur + `emit_use` produisent le **même Rust** AVANT de
toucher aux décisions. Ensuite seulement, on améliore les **décisions** (dans le moteur), et `emit_use` (figé)
suit automatiquement. C'est ça « travailler intelligemment » : zéro big-bang, zéro régression.

## Carte des vagues

| Vague | Piste | Réalise | Fichiers POSSÉDÉS | Parallèle ? |
|:--:|:--:|---|---|---|
| **1** | 1.1 | [30800] | `ir.rs` (schéma figé `OwnershipPlan`/`Disp`) | séquentiel (leaf) |
| **2** | 2.1 | [30800][40500] | `resolve.rs` (moteur → plan, **reproduit l'existant**), `lower.rs` (`emit_use`), `driver.rs` (signature) | séquentiel — **socle NEUTRE (gate diff-vide)** |
| **3** | 3.1 | [30800] | `resolve.rs` (exploitation : **modes de méthode** + **last-use réel**) | **‖** avec 3.2 |
| **3** | 3.2 | [40500] | `parse.rs` (**désucrage opérateurs** `a OP b`→`a.__op__(b)`) | **‖** avec 3.1 |
| **4** | 4.1 | toutes | unification + **gate raytracer** (clones résiduels éliminés + perf) + corpus + polish | séquentiel — **gate final** |

## Graphe de dépendances

```
1.1 ─► 2.1 (NEUTRE, diff-vide) ─┬─► 3.1 ─┐
                                 └─► 3.2 ─┴─► 4.1 (gate)
```
- **1.1** fige le schéma `OwnershipPlan`.
- **2.1** = le socle **neutre** : `resolve` produit le plan en **reproduisant** les décisions actuelles
  (modes V16, moves single-use, for-élément, méthodes par valeur comme aujourd'hui) ; `lower.emit_use` consomme
  le plan en reproduisant le Rust actuel. **DoD = diff Rust VIDE** (corpus+raytracer). Prouve la neutralité.
- **3.1 ‖ 3.2** : exploitation. `resolve` (3.1) **améliore les décisions** (méthodes → `&`, last-use réel) →
  `emit_use` suit. `parse` (3.2) désucre les opérateurs → leurs opérandes passent par `emit_use`. Fichiers
  disjoints → **parallèles**.
- **4.1** : gate final.

## L'invariant central (le critère de « fini »)

1. **Vague 2** : **diff Rust VIDE** sur corpus 80 + raytracer (`ironc emit` avant/après → identique). C'est la
   preuve que le socle est **neutre** avant toute exploitation. + corpus vert + 4 suites tests.
2. **Vagues 3-4** : à chaque greffe, **corpus 80 reste VERT** ; le raytracer perd ses clones résiduels
   (`r.clone()` par objet, `rec.clone()`) **gratuitement** (méthodes/last-use sont dans le moteur) ; `a+b` sur
   valeur empruntée compile.
3. **Gate 4.1** : raytracer `emit→build→run`→image correcte + **perf ≥ V16** (re-mesurée) ; corpus 80 vert.

> **Contrat de sûreté (rappel cadrage)** : optimiste → rétrogradation **monotone** (terminaison) ; **dans le
> doute → Clone/ByValue** (toujours correct). Un `&T` qui casserait `rustc` = bug bloquant.

## Test mental de non-collision (validé)

- 3.1 `resolve.rs` · 3.2 `parse.rs` → **disjoints**, parallélisables. 2.1 (resolve+lower+driver) séquentiel
  avant. 4.1 (lower polish + validation) après. ✅

> **Méthode.** Planif figée ici. **Prochaine étape = réalisation sur ton feu vert** (autonomie de phase 3 :
> socle neutre → exploitation → gate, sans m'arrêter entre vagues ; seul gate = 4.1 ou vrai blocage).
