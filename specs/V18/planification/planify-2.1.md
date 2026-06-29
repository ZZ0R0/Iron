# planify-2.1 — Closures de 1ʳᵉ classe / fonctions d'ordre supérieur

- Réalise (cadrage) : [21000]
- Version : V18 · Vague : 2 · Piste : 1 · Statut : todo
- Dépend de : planify-1.1/1.2 (la garde protège ce refactor de cœur)

## But
Rendre les closures passables en argument à des **fns utilisateur**, retournées, stockées ; lowering
`impl Fn` (défaut, coût nul) ou `Box<dyn Fn>` (stocké/hétérogène). Capacité-pivot (l'async en dépend).

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/infer/` (inférence du type fonction `fn(A)->B` propagé depuis le site d'appel).
- `ironc/src/lower.rs` (lowering closure-arg, closure-retour, closure-stockée).
- `ironc/src/ir.rs` (si un type/attribut est nécessaire pour marquer une fn-param générique).

## Interface produite / consommée
**Consommée** : le moteur I15 (propagation de type). **Produite** : une fn utilisateur à param fonction →
`fn f<F: Fn(A)->B>(g: F, …)` ; closure stockée/retournée → `Box<dyn Fn(A)->B>` (capture `move` des possédés).

## Étapes
1. **Inférence** : un param appelé comme une fonction (`f(x)` dans le corps) → type `fn(typeof x)->typeof f(x)`,
   propagé depuis le site d'appel (lambda passée).
2. **Lowering arg** : fn à param fonction → générique borné `F: Fn(...)` ; l'argument lambda → closure Rust.
3. **Lowering stocké/retourné** : `Box<dyn Fn(...)>` ; capture `move` des variables possédées (sémantique valeur).
4. Critère **mono vs Box** : générique par défaut ; `Box<dyn Fn>` si stocké en collection/champ ou retour hétérogène.

## Definition of Done (vérifiable, via [70400])
- `apply(lambda n: n*2, 21)` → **42** ; `adder(10)(5)` → 15 (closure retournée, type explicite) ; une closure
  stockée dans une liste rappelée fonctionne. Cas ajoutés au corpus → `cargo test` vert.
- Corpus [70400] **reste vert** (zéro régression). 0 `unsafe`.

## Notes
- **Refactor de cœur séquentiel** (pas de sous-agents). La garde [70400] capte toute régression.
- Durées de vie de capture non triviales → capture `move`/`shared` ou diagnostic (le 0.1%).
