# planify-4.1 — Unification : gate raytracer (clones résiduels éliminés) + corpus + perf

- Réalise (cadrage) : toutes ([30800] · [40500])
- Version : V17 · Vague : 4 · Piste : 1 · Statut : todo
- Dépend de : planify-3.1, 3.2 (greffe des exploitations)

## But
Greffer toutes les pistes, **re-valider l'e2e RÉEL** et confirmer que le moteur d'ownership livre : Rust de
raytracer **sans clones résiduels**, qui compile, tourne, produit la bonne image, **plus vite** que V16. Plus
le polish stdlib (§2.7 du rapport : `int`/`**`) si non déjà fait.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/lower.rs` (polish §2.7 éventuel) — petit.
- `specs/V11-test-2/CARTE-80.md` / un `ETAT` V17 (consignation mesurée).

## Étapes
1. **Corpus 80** : re-jouer la batterie — zéro régression compile/run (les programmes qui passaient passent).
2. **Raytracer e2e** : `ironc emit` → `cargo build --release` → `./iron_out` → `output.png`. Vérifier :
   (a) **plus aucun clone scene-size ni O(objets)** dans le Rust généré (`grep .clone()` sur le hot path) ;
   (b) image **correcte** ; (c) **temps mesuré** vs V16 (gain ≥ 0, idéalement les `r.clone()`/`rec.clone()` retirés).
3. **Polish §2.7** (si reste) : `int(f64)`→cast, `f64 ** n`→`powi/powf`.
4. Consigner : socle ownership unifié OK, perf raytracer (M-rays/s), corpus N/80, frictions levées.

## Definition of Done (vérifiable)
- **Gate e2e** : corpus 80 vert **ET** raytracer `output.png` correct **ET** ≥ aussi rapide que V16 (mesuré) **ET**
  hot path sans `.clone()` résiduel non justifié.
- `cargo test -p ironc` vert ; 0 `unsafe` (I2).

## Notes
- **Gate de fin de V17.** Si un clone résiduel demeure (cas non couvert par le moteur), on **ré-itère** sur 3.1
  avant de clore (boucle de vérité). C'est le moteur qui doit décider — pas un patch dans `lower`.
- Le socle étant unifié, tout nouveau besoin ownership futur se branche **dans le moteur**, plus en règle ad-hoc.
