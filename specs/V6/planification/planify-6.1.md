# [6.1] Unification + gate V6

- Spec(s) : toutes · V6 · **Vague 6** (séquentielle) · Statut : todo · Dépend de : 2.x–4.x

## But
Greffer toutes les pistes V6, revalider l'**e2e RÉEL** = gate V6 (`V6/README.md`).

## Fichiers possédés
Aucun module exclusif : intégration + corrections ciblées (sans toucher les contrats gelés de 1.1).

## Étapes
1. Brancher toutes les features (I11, Rc, closures, motifs, ownership, source-maps expr, interop, modules, stdlib, comparaison chaînée).
2. Rejouer le **corpus 22** (non-régression) + les programmes-preuves de chaque spec.
3. Vérifier zéro `unsafe` + `cargo test`.

## Critère de done (= gate V6)
Chaque spec a sa preuve exécutée · corpus 22 inchangé · zéro `unsafe` · `cargo test` vert.
Faiblesses/dettes → `V6/realisation.md` (et éventuelle V7).

## Notes
Invariant dur : si une piste casse le corpus, ré-unifier avant de continuer.
