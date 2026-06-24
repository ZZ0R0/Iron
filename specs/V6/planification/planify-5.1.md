# [5.1] Corpus de validation — métrique de vérité (90100)

- Spec parente : 90100 · V6 · **Vague 5** (méta) · Statut : todo · Dépend de : toutes (2.x–4.x)
- **DIFFÉRÉ** : à lancer une fois le langage **gelé** (sinon cible mouvante).

## But
Chiffrer la correction `Iron→Rust` + le coût clone-fallback sur un **corpus large** (≫ 22).

## Fichiers possédés
`corpus/` (programmes Iron + sortie attendue), un **runner** (`ironc`-driven), un script d'agrégation.

## Étapes
1. Constituer le corpus (à la main et/ou génératif) avec oracle de sortie.
2. Runner : `ironc run` chaque cas → comparer ; agréger correction / densité de clones / taux de compilation.
3. Publier le rapport ; **refuser** de chiffrer sous le seuil de corpus.

## Critère de done
Harnais sur ≥ N cas + rapport reproductible (N à fixer).

## Notes
Dernier de V6 (dépend du gel du reste).
