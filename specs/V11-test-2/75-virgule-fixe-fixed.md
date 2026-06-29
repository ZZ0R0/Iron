# 75 — Arithmétique en virgule fixe
> **Domaine** Calcul numérique · **Crate(s)** `ironc add fixed` · **Risque Iron** 🟥 · **Difficulté** élevée

## But
Calculer avec des nombres à **virgule fixe** (ex. `I16F16` : 16 bits entiers, 16 fractionnaires) :
additions/multiplications déterministes, conversion vers/depuis f64.

## Ce que ça teste dans Iron
`fixed` encode la position de la virgule **au niveau type** via const-génériques/typenum
(`FixedI32<U16>` / `I16F16`). Cible directe de [50600] C3 (const au niveau type) sur un type numérique.
Teste les alias monomorphes (`I16F16`) + opérateurs.

## Approche
Cibler l'alias `I16F16` (monomorphe) : `I16F16.from_num(x)` (ou `from_str`), `+`/`*`, `to_num::<f64>()`.
L'alias évite d'exposer les const-génériques nus.

## Done
`ironc run` imprime un produit en virgule fixe converti en f64 (valeur attendue).
