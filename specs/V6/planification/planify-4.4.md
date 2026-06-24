# [4.4] Comparaison chaînée `a < b < c` (10300)

- Spec parente : 10300 · V6 · **Vague 4** · Statut : todo · Dépend de : 1.1
- (`parse.cmp_expr` **isolé** → parallélisable.)

## But
`a < b < c` ≡ `a < b and b < c` (aujourd'hui parsé `(a<b)<c`, erreur de type).

## Fichiers possédés
`src/parse.rs` (`cmp_expr`).

## Étapes
1. Dans `cmp_expr`, collecter une chaîne `a op b op c …` → AST conjonctif (`and`).
2. Évaluation **unique** des opérandes intermédiaires (`let` temporaire si effet de bord).

## Critère de done
`if 1 < x < 10:` compile + exécute (vrai dans l'intervalle) ; corpus 22/22 inchangé.

## Notes
Opérande à effet de bord → `let` temporaire (spec 10300).
