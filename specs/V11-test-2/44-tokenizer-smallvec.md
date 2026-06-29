# 44 — Tokenizer à petit vecteur inline
> **Domaine** Structures de données · **Crate(s)** `ironc add smallvec` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Découper une chaîne en tokens stockés dans un `SmallVec` (inline jusqu'à N, puis heap), compter les tokens.

## Ce que ça teste dans Iron
`SmallVec<[T; N]>` mêle **type tableau à taille fixe** et générique d'élément dans le paramètre. Teste la
projection d'un paramètre de type qui est lui-même un `[T; N]` (const-générique implicite).

## Approche
Cibler `SmallVec` par défaut : `SmallVec.new()`, `push`, `len`, itération. La capacité inline est dans le
type ; mesurer si le binder l'exprime ou si une ascription est requise.

## Done
`ironc run` tokenise une phrase et imprime le nombre de tokens attendu.
