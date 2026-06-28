# 53 — Grands entiers (factorielles / Fibonacci énormes)
> **Domaine** Maths/vectoriel · **Crate(s)** `ironc add num-bigint` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Calculer `1000!`, `fib(10000)` ou `2^4096` en précision arbitraire, et imprimer le résultat décimal
exact (des centaines/milliers de chiffres). En appui : pgcd, exponentiation modulaire.

## Ce que ça teste dans Iron
Cas **feuille** idéal pour le binder auto : `BigUint` / `BigInt` sont des types **concrets** avec
surcharge d'opérateurs native (`+ - * % pow`). On teste surtout la **surcharge d'opérateurs (V11)** sur un
type **non primitif** — `acc *= n` doit se router vers `BigUint::mul_assign` — la conversion
`u64 → BigUint`, et le `Display` de très grands nombres. C'est le projet **témoin** du binder.

## Approche
`ironc add num-bigint`, `import`, puis boucle débutante : un accumulateur `BigUint` multiplié dans une
boucle (`factorielle`), ou deux accumulateurs additionnés (`Fibonacci`). Initialiser via
`BigUint::from(1u32)` plutôt que par les traits `One`/`Zero`. Lecture/écriture par `to_string()` et
`parse()`. Si **lui** ne passe pas, le binder a un problème de base.

## Risque / limites attendues
- Les constantes passent par les **traits `num-traits`** (`One::one()`, `Zero::zero()`) : préférer
  `BigUint::from(...)`, que le binder mappe sans ambiguïté.
- `pow` attend un exposant `u32` ; l'exponentiation **modulaire** est une méthode (`modpow`) à exposer.
- Aucun écueil de génériques ni d'état mutable global : risque réellement faible.

## Done
`ironc run` imprime `100!` (158 chiffres) exact et `fib(1000)`, vérifiés contre une valeur de référence.
`acc *= n` compile via la surcharge d'opérateurs V11. Aucun `unsafe`.
