# 28 — Buffer à capacité fixe (sans allocation dynamique)
> **Domaine** Structures de données · **Crate(s)** `ironc add arrayvec` · **Risque Iron** 🟡 · **Difficulté** élevée

## But
Utiliser un `ArrayVec<T, CAP>` (capacité fixe, pile) : pousser des éléments, dépasser la capacité
proprement (Result/erreur), itérer, calculer une somme.

## Ce que ça teste dans Iron
Le **const-générique de capacité** `ArrayVec<T, const CAP: usize>` — exactement la cible de [50600] C3
(`ArrayVec[u8, 16]` → `ArrayVec::<u8, 16>`). Teste le portage d'une valeur const au niveau type via
ascription, sans alias dédié (contrairement aux dimensions nalgebra de C1).

## Approche
Ascription `v: ArrayVec[i64, 8] = ArrayVec.new()` ; `push` ; `len` ; somme par itération. La taille 8 doit
être pinnée par l'ascription pour que rustc infère le const-générique.

## Done
`ironc run` pousse N≤8 éléments et imprime la somme ; un push au-delà de la capacité est géré proprement.
