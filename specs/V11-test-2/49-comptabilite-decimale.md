# 49 — Comptabilité en décimal exact
> **Domaine** Finance · **Crate(s)** `ironc add rust_decimal` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Additionner des montants décimaux sans erreur de flottant (`0.1 + 0.2 = 0.3` exact), arrondir, formater.

## Ce que ça teste dans Iron
`Decimal` est un type non-générique mais à **construction par chemins multiples** (`from_str`, `new(mantisse,
échelle)`, macro `dec!`) et **opérateurs surchargés**. Teste le binding d'un type numérique custom et le
mappage des opérateurs Iron sur ses impls.

## Approche
`Decimal.from_str("0.1")` + `+` + `round_dp(2)` + `to_string`. Vérifier l'exactitude vs flottant.

## Done
`ironc run` imprime `0.1 + 0.2 = 0.3` (exact) et un total arrondi correct.
