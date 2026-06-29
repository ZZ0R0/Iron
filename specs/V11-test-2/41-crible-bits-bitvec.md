# 41 — Crible d'Ératosthène compact (bits)
> **Domaine** Structures de données · **Crate(s)** `ironc add bitvec` · **Risque Iron** 🟥 · **Difficulté** élevée

## But
Calculer les nombres premiers jusqu'à N avec un tableau de **bits** compact (crible), compter les premiers.

## Ce que ça teste dans Iron
`bitvec` encode l'**ordre des bits et le stockage au niveau type** (`BitVec<T, O>` avec `O: BitOrder`).
API à génériques de marqueur + indexation bit-à-bit. Teste un crate type-level orienté performance.

## Approche
Cibler `BitVec` par défaut (`Lsb0`, `usize`) : `BitVec.repeat(true, n)` ou `bitvec!`-équivalent via ctor,
`set(i, false)`, `count_ones()`. À défaut, mesurer la couverture + rapport.

## Done
`ironc run` imprime le nombre de premiers < 100 (= 25), ou couverture mesurée + rapport si muré.
