# [3.1] Unification + gate V1

- Spec parente : 30200 · V1 · **Vague 3** (séquentielle) · Statut : todo
- Dépend de : 2.1, 2.2

## But
Greffer escape analysis + lowering emprunts, et **valider le gate V1** (optimisation pure).

## Fichiers possédés
Câblage `driver`/`resolve`/`lower` ; corrections d'intégration (sans casser les contrats).

## Étapes
1. Brancher `resolve` réel → `lower` réel.
2. Rejouer **le corpus V0 entier** (`ironc/tests/fixtures/`) : sorties **identiques** à V0 (diff vide).
3. Compter les `.clone()` du Rust généré : **strictement `<`** vs V0.
4. Vérifier `--audit` + zéro `unsafe` + « compile toujours » (I1).

## Critère de done (= gate V1)
Corpus **11/11 sorties identiques** · `.clone()` strictement réduit · `--audit` opérationnel · zéro `unsafe`.
Tout écart (emprunt cassant I1) → corrigé (repli clone) avant de clore.

## Notes
Invariant dur : si un emprunt casse la compilation d'un programme V0-valide, **repli sur clone** (I1) puis
on note la cause. Faiblesses → `V1/realisation.md` → cadrage V2.
