# planify-2.1 — Le socle NEUTRE : moteur → `OwnershipPlan` + `emit_use`, diff Rust VIDE

- Réalise (cadrage) : [30800] (moteur) + [40500] (consommation)
- Version : V17 · Vague : 2 · Piste : 1 · Statut : todo
- Dépend de : planify-1.1 (schéma figé)

## But
Poser le socle **sans rien changer au Rust généré** : `resolve.rs` produit un `OwnershipPlan` qui **reproduit
exactement** les décisions actuelles (modes V16, moves single-use, for-élément, méthodes par valeur comme
aujourd'hui) ; `lower.rs` consomme le plan via **un seul** `emit_use()` qui **reproduit** le Rust actuel. La
preuve = **diff Rust VIDE** (corpus + raytracer). C'est la fondation : tout passe désormais par le plan, sans
régression, **avant** d'exploiter.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/resolve.rs` (refonte : les deux analyses → un `OwnershipPlan`).
- `ironc/src/lower.rs` (`emit_use()` ; bascule des sites sur le plan).
- `ironc/src/driver.rs` (signature `resolve` → `OwnershipPlan`).

## Interface produite / consommée
**Consommée** : `OwnershipPlan`/`Disp` (1.1) ; l'AST + `TypeMap`. **Produite** : `resolve(ast, types) ->
OwnershipPlan` ; `lower::emit_use(node, expr) -> TokenStream` piloté par `plan.sites`/`plan.params`.

## Étapes
1. **`resolve` → `OwnershipPlan`** : fusionner l'escape interprocédurale (V16, → `params`) et `compute_moves`
   (→ `sites` avec `Move`/`Clone`) + le for-élément (`for_var_borrowable`) en **un** plan. Les décisions
   restent **identiques à l'actuel** (on ne change pas encore les règles — on les **regroupe**).
2. **`emit_use()`** dans `lower` : `Borrow→&place`, `BorrowMut→&mut place`, `Move→value`, `Clone→value.clone()`.
3. **Basculer les sites** sur `emit_use` en **reproduisant** le comportement courant : args d'appel (libre +
   méthode + externe via la coercion conservée), récepteur, for-élément, retours, champs, captures. Les méthodes
   restent **par valeur** (comme aujourd'hui — la neutralité l'exige ; l'exploitation viendra en 3.1).
4. `driver.rs` : adapter la signature.
5. **Vérifier la NEUTRALITÉ** : `ironc emit` sur le corpus + le raytracer **avant/après** → **diff vide**.

## Definition of Done (vérifiable)
- **Diff Rust VIDE** : pour le corpus 80 + le raytracer, le `main.rs` généré est **identique** à l'actuel
  (comparaison automatisée avant/après sur un échantillon large + le raytracer complet).
- `cargo build -p ironc` + `cargo test -p ironc` verts ; corpus échantillon run inchangé ; 0 `unsafe`.

## Notes
- **C'est LE gate de la fondation** : on ne touche aux **décisions** qu'après avoir prouvé la neutralité du
  **plumbing**. Commits intermédiaires ; si un diff apparaît, c'est un bug de reproduction à corriger AVANT
  d'avancer.
- Risque rayon de souffle maximal — d'où la neutralité comme garde-fou absolu.
