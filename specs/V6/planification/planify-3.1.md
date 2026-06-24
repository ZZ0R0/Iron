# [3.1] Inférence des types de paramètres (20500)

- Spec parente : 20500 · V6 · **Vague 3** · Statut : todo · Dépend de : 1.1

## But
Ne plus exiger l'annotation de type de paramètre quand il est déductible (dette 20100).

## Fichiers possédés
`src/infer/` (passe d'inférence des params).

## Étapes
1. Inférer un type de param depuis son usage local (ops, champs, appels) et les sites d'appel.
2. Repli : param non résolu → **générique implicite** (`fn f(x)` → `fn f[T](x: T)`, réutilise 20300).
3. Annotation explicite prime.

## Critère de done
Programmes sans annotation de param (là où déductible) → compilent ; corpus 22/22 inchangé.

## Notes
Ambiguïté numérique → générique ou défaut `i64` (cf. spec 20500 risques).
