# planify-5.1 — Unification + mesure de self-suffisance (le chiffre)

- Réalise (cadrage) : toutes ([70400]·[21000]·[21800]·[21300])
- Version : V18 · Vague : 5 · Piste : 1 · Statut : todo
- Dépend de : planify-2.1, 3.1, 4.1 (greffe des 3 capacités/ergonomie)

## But
Greffer toutes les pistes, **re-jouer la suite [70400] en entier**, et **chiffrer la self-suffisance** : N
programmes réels (raytracer, closures, async, ergonomie) compilent **et** produisent la bonne sortie. Plus la
mise à jour de la doc (`docs/`) et de la carte corpus.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/tests/corpus/` (cas finaux : closures, async, raytracer-parallèle, ergonomie).
- `docs/*` (mettre à jour `gotchas.md` — les frictions levées —, `status.md`, `language.md` — closures/async).
- Un `ETAT` V18 (consignation mesurée).

## Étapes
1. **Re-jouer [70400]** sur tout le corpus (noyau + éventail + cas des 3 axes) → `cargo test` vert.
2. **Cas de validation des axes** : higher-order (`apply`), closure stockée/retournée, async simple,
   `thread.scope(&scene)` du raytracer, `main` avec `?`, `int/**`, raytracer-sans-annotations.
3. **Chiffrer** : « N/N cas verts » = la **métrique de vérité** de la self-suffisance.
4. **Doc** : retirer de `gotchas.md` les frictions levées (`?`-main, `.copy()` si last-use, opérateurs si
   désucrés) ; documenter closures + async dans `language.md` ; mettre à jour `status.md`.

## Definition of Done (vérifiable)
- **Gate** : `cargo test` (corpus complet) vert ; le **chiffre N/N** consigné ; le raytracer parallèle via
  `thread.scope` tourne + image correcte.
- `docs/` à jour (gotchas levés, closures/async documentés). 0 `unsafe`.

## Notes
- **Gate de fin de V18.** Si un cas régresse, on **ré-itère** sur l'axe concerné (la garde le pointe) avant de clore.
- C'est ici qu'on **mesure** « suffisant à lui-même » : un chiffre, pas un ressenti.
