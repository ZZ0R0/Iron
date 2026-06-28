# 55 — Simulation physique 2D (balles qui rebondissent)
> **Domaine** Maths/vectoriel · **Crate(s)** `ironc add rapier2d` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Simuler quelques balles soumises à la gravité, rebondissant dans une boîte, et imprimer leurs positions
à chaque pas de temps (ou après N pas). C'est la **boucle de simulation** mutable classique.

## Ce que ça teste dans Iron
La **frontière dure** côté maths : `rapier2d` repose sur un **monde mutable** et une **boucle `step()`**.
Un pas emprunte **simultanément ~10 ensembles d'état en `&mut`** (`RigidBodySet`, `ColliderSet`,
`IslandManager`, `BroadPhase`, `NarrowPhase`, `PhysicsPipeline`, solveurs…). Ce motif heurte de plein
fouet la **sémantique par valeur** et le **« pas d'état global mutable »** d'Iron. Les corps se créent par
**builders fluents** (`RigidBodyBuilder::dynamic().translation(..).build()`) que le binder ne modélise pas.

## Approche
Le binder auto **échoue** ici : la signature de `step()` (multi-`&mut`) et les builders sont inexprimables
tels quels. Voie réaliste : écrire un **petit crate-façade Rust** encapsulant tout le monde dans **un seul
objet opaque** possédé par Iron, exposant `new_world(gravity)`, `add_ball(x, y)`, `step(dt)`,
`position_of(handle) -> (f64, f64)` — puis binder **cette façade**, pas `rapier` directement. Les
`Vector2`/`Isometry` de nalgebra fuient dans l'API → génériques à aplatir en tuples.

## Risque / limites attendues
- **Mur principal** : `step()` exige plusieurs `&mut` à la fois ; sans façade, impossible.
- Sous sémantique par valeur **stricte**, chaque pas clonerait le monde → **catastrophe perf** (warning
  clone-fallback I4). Il faut un objet à `&mut self` unique pour que ce soit viable.
- Builders fluents + handles (`RigidBodyHandle`) opaques : modélisables seulement via la façade.

## Done
Avec une façade minimale, `ironc run` simule N pas et imprime des trajectoires plausibles (les balles
retombent et rebondissent). **Sans** façade : on **documente l'échec** du binder auto comme résultat.
