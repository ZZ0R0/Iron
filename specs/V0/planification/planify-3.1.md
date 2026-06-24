# [3.1] Unification + e2e réel (gate V0)

- Spec(s) parente(s) : toutes · V0 · **Vague 3** (séquentielle) · Statut : todo
- Dépend de : 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7

## But
Greffer tous les passes **réels** sur le squelette, retirer les stubs, et revalider l'**e2e RÉEL** = le gate V0 (`V0/README.md §6`).

## Fichiers possédés
Aucun module exclusif : **câblage** (`driver`/`main`) + corrections d'intégration ciblées (jamais de réécriture des contrats gelés).

## Interface
Branche la chaîne réelle : `lex → parse → infer → resolve → lower → driver`, avec `sourcemap` sur la sortie `cargo`.

## Étapes
1. Remplacer les stubs par les passes réels (2.1–2.7).
2. Faire passer le **programme représentatif** (fn+struct+`if/while/for`+récursion+`list/dict`+`Option/Result`+`?`) **dans les 2 syntaxes de bloc**.
3. Vérifier la remontée source-map e2e (erreur `rustc` forcée → `Span` Iron).
4. Corriger les trous d'intégration (sans toucher `ir.rs`).

## Critère de done (= gate V0)
`ironc run` représentatif → `cargo` **vert** → **sortie correcte** ; `#![forbid(unsafe_code)]` présent ; erreur `rustc` → **bonne ligne Iron** ; déterminisme ; **les 2 syntaxes** marchent.

## Notes
Invariant dur : si une piste casse l'e2e, **ré-unifier avant de continuer**. Faiblesses/dettes constatées → notées pour le cadrage V1 (phase 4).
