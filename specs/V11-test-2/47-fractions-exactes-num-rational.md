# 47 — Calcul exact en fractions
> **Domaine** Maths · **Crate(s)** `ironc add num-rational` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Additionner/multiplier des fractions exactes (`1/3 + 1/6 = 1/2`), réduire, comparer, convertir en f64.

## Ce que ça teste dans Iron
`Ratio<T>` est un **générique scalaire** (comme num-complex/geo, cible [24100]/C1-like). Teste la
construction `Ratio.new(num, den)`, les opérateurs surchargés (`+`/`*`) et l'accès `numer()`/`denom()`.

## Approche
Cibler `Rational64` (= `Ratio<i64>`, alias monomorphe) : `Ratio.new(1, 3)`, `+`, `to_f64`. L'alias
monomorphe évite le gros des génériques (proche du levier C1).

## Done
`ironc run` imprime `1/3 + 1/6` réduit en `1/2` et sa valeur f64 (0.5).
