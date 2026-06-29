# planify-4.2 — C3 : const-génériques portés dans le modèle de types

- Réalise (cadrage) : [50600] C3
- Version : V15 · Vague : 4 · Piste : 2 · Statut : todo
- Dépend de : planify-3.1 (binder modularisé) — **‖ planify-4.1** (fichiers disjoints)

## But
Porter les **valeurs au niveau type** (`Const<N>`, littéraux const) à travers tout le pipeline binding→lowering,
pour exprimer les shapes que C1 (alias) ne couvre pas (fn libres `f<const N: usize>()`, types paramétrés par
une valeur sans alias dédié). Complète C1 ; résorbe le `ConstGenericShape` résiduel hors-alias.

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/bind/mapty.rs` (reconnaissance `Const<N>` → `ExternArg::Const`).
- `ironc/src/lower.rs` (lowering du turbofish const).
- `ironc/src/iri.rs` (syntaxe `.iri` d'un arg const — **section parse uniquement**, disjointe du two-layer de 3.1).

## Interface produite / consommée
**Consommée** : `Type::ExternApp(String, Vec<ExternArg>)` avec `ExternArg::Const(String)` (1.1).
**Produite** : `map_type` reconnaît `Const<N>`/typenum-littéral comme `ExternArg::Const` ; `lower` émet
`path::<.. , N>` (turbofish mixant types et consts) ; `iri.rs` parse `Foo[N]` côté const.

## Étapes
1. `mapty.rs` : dans la branche resolved_path, reconnaître un argument `Const<N>` (et littéral const) → produire
   `ExternArg::Const("N")` au lieu d'un type opaque.
2. `lower.rs` : étendre le lowering `ExternApp` pour émettre un turbofish qui **mixe** `ExternArg::Ty` et
   `ExternArg::Const` dans le bon ordre (`Foo::<T, 3>`).
3. `iri.rs` : parser un argument const dans `Name[...]` (un identifiant/nombre traité comme const).
4. Cas non exprimable → `SkipReason::ConstGenericShape` (pas de drop).

## Definition of Done (vérifiable)
- **Run réel** : un usage const-générique **hors-alias** (ex. une fn `f::<N>()` ou un type `Foo<3>`) compile +
  produit le bon résultat.
- Test de lowering : `ExternApp("p::Foo", [Ty(i32), Const("3")])` → `p::Foo::<i32, 3>` (test unitaire, parallèle
  au `bridge_abi_tests` existant).
- e2e binding VERT ; couverture stable ou en hausse ; 0 `unsafe`.

## Notes
- **N'édite jamais `impls.rs`** → parallèle strict avec 4.1.
- `lower.rs` est **partagé** avec C4 (6.1) → 6.1 est **séquencé après** cette piste (pas en parallèle).
- Le cas **octets** (`GenericArray<u8, _>`) est déjà couvert par `@arr` (V13) — C3 vise les shapes **non-octets**
  (dimensions résiduelles, tailles génériques).
