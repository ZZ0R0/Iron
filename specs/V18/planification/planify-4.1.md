# planify-4.1 — Async / concurrence : `thread.scope(&data)` + async/await consolidé

- Réalise (cadrage) : [21300]
- Version : V18 · Vague : 4 · Piste : 1 · Statut : todo
- Dépend de : planify-2.1 (closures — les lambdas de worker) ; garde [70400]

## But
Donner un modèle concurrent **usable et borné** : `thread.scope` pour capturer `&data` en lecture (le cas
raytracer, sans clone — **bénéfice direct de l'ownership V17**) ; `async`/`await` consolidé (runtime opt-in).
L'async exotique reste `rust{}`.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/lower.rs` (lowering `thread.scope`, `async`/`await`, capture `&` partagée).
- `ironc/bindings/std.iri` (primitives `thread.scope`/canaux si besoin).

## Interface produite / consommée
**Consommée** : closures de 1ʳᵉ classe (2.1) pour les lambdas worker ; l'ownership V17 (la donnée est empruntée).
**Produite** : `thread.scope(lambda s: …)` → `std::thread::scope(|s| …)` capturant `&data` ; `async def`→`async fn`
+ `#[tokio::main]` (si async présent).

## Étapes
1. **(b) `thread.scope`** d'abord (le plus utile pour le calcul) : binder/abaisser `std::thread::scope` ; les
   workers capturent `&scene` (lecture partagée) **sans** reconstruire — résout §2.6 du rapport.
2. **(a) async/await** : consolider `async def`/`await` (existants) ; runtime tokio injecté **seulement si**
   `async` présent (I5) ; un programme async simple tourne.
3. **(c) canaux** (mpsc) : seulement si un cas du corpus le demande (surface minimale).

## Definition of Done (vérifiable, via [70400])
- **Le raytracer parallélise via `thread.scope(&scene)`** — **sans** reconstruire scène/caméra par worker ;
  image correcte ; perf ≥ la version threads actuelle.
- Un programme `async` simple compile + run. Corpus [70400] vert ; 0 `unsafe`.

## Notes
- **Le plus gros lift de V18** ; périmètre **borné** (commit à scope+async-basique+shared_sync). Hors-périmètre
  → `rust{}`, assumé.
- Synergie : dépend des closures (2.1) ; à faire après.
