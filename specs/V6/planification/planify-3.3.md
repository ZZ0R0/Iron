# [3.3] Ownership avancé (30400)

- Spec parente : 30400 · V6 · **Vague 3** · Statut : todo · Dépend de : 1.1

## But
Escape analysis **point-fixe**, **move-hors-struct** (retour de champ générique), **scoping `for`** post-boucle.

## Fichiers possédés
`src/resolve.rs` (point-fixe + détection move-hors-struct), `src/lower.rs` (for-var hoisting, move partiel).

## Étapes
1. Itérer `ParamModes` jusqu'à stabilité (emprunt transitif `ByRef`).
2. Param dont un champ est **retourné par valeur** → `ByValue` + `return p.a` move partiel.
3. For-var utilisée après la boucle → hoistée (comme une locale).

## Critère de done
`fn first[T](p) -> T: return p.a` compile ; moins de clones (emprunt transitif) ; for-var post-boucle compile ; corpus 22/22 inchangé.

## Notes
Move-hors-struct partiel : sûr seulement si le struct n'est pas réutilisé — border l'analyse.
