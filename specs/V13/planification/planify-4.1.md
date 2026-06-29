# planify-4.1 — Dimensions de matrices (`Const<N>`/typenum dims) → nalgebra, ndarray ⚠️ DUR/PARTIEL

- Réalise (cadrage) : [24500] sous-cas B
- Version : V13 · Vague : 4 · Piste : 1 · Statut : todo (**gate-partiel assumé**)
- Dépend de : planify-1.1 (octets ; B réutilise la machinerie const)

## But
Binder les **constructeurs par-dimension** des alias matrices/vecteurs (`Vector3::new(x,y,z)`, `Matrix3::new(…)`)
là où le binder ne voit qu'un `Matrix<T, Const<3>, Const<1>>` opaque. **Honnête : pas de solution générale —
table d'alias-dimension bornée.**

## Fichiers POSSÉDÉS
- `ironc/src/bind.rs` : table d'alias-dimension connus (`Vector2..6`, `Matrix2..6`, `Point2/3`) → exposer leurs
  ctors `new(x: T, …)` avec la bonne arité ; résoudre `Const<N>`/typenum `U{N}` dans la signature.
- `ironc/src/ir.rs` : `Type::ConstN(u64)` **si** une taille doit être émise explicitement (sinon inutile).
- `ironc/src/lower.rs` : `ConstN(N)` → `N` (const-générique) / `typenum::U{N}` (typenum), selon la position.

## Interface produite / consommée
- **Consomme** : ExternApp (V12), B33 (alias), `@arr` (V13 1.1).
- **Produit** : ctors dimension-explicites bindés (pour les dims tabulées).

## Étapes
1. Identifier dans rustdoc les ctors par-dimension (associés à l'alias ou à l'instanciation Matrix concrète).
2. Table d'alias → arité + dims ; émettre `Vector3.new(x,y,z) -> Vector3[T]`.
3. Lowering : monomorphiser (`Vector3` = `Matrix::<T,Const<3>,Const<1>,…>` ou l'alias direct si rustc l'accepte).
4. Tester `Vector3.new(1.0,2.0,3.0).norm()` ; idem ndarray `Array` si faisable.

## Definition of Done (vérifiable)
1. nalgebra : `Vector3.new(1.0,2.0,3.0).norm()` → `√14` (run réel) **si** la table aboutit.
2. **Sinon** : limite documentée précisément (quelle étape bloque : alias, ctor par-dim, ou monomorphisation).
3. Gate VERT 24/0 ; 0 `unsafe`.

## Notes
- **Risque assumé (cadrage §1bis)** : au-delà des dims tabulées, reste bloqué. ndarray (`Array2`/`Dim`) et
  rustfft (`dyn Fft` + `&mut [Complex]`) ont une complexité **supplémentaire** (storage/dyn/out-slice) — peuvent
  rester partiels même avec la table. Ne pas survendre la généralité.
