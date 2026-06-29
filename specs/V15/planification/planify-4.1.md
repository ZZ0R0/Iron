# planify-4.1 — C1 : alias monomorphisé → type nominal (récupère les ctors par-dimension)

- Réalise (cadrage) : [50600] C1
- Version : V15 · Vague : 4 · Piste : 1 · Statut : todo
- Dépend de : planify-3.1 (binder modularisé + reverse-alias exposée) — **‖ planify-4.2** (fichiers disjoints)

## But
Faire qu'un `impl` inhérent porté par une **monomorphisation concrète** (`impl<T> Matrix<T, Const<3>,
Const<1>, …>`) pour laquelle rustdoc expose un **alias** (`Vector3<T>`) attache ses ctors/méthodes au **type
nominal de l'alias**, pas à la base générique (où ils collisionneraient). Résorbe la classe
`ConstGenericShape` pour les ctors par-dimension **sans nommer aucun crate** (la règle lit l'alias + l'impl
depuis rustdoc).

## Fichiers POSSÉDÉS (ownership disjoint)
- `ironc/src/bind/impls.rs` **uniquement**. (Lecture seule de `mapty::alias_for_monomorphization`, figée en 2.1.)

## Interface produite / consommée
**Consommée** : `mapty::alias_for_monomorphization(self_ty: &Json) -> Option<String>` (2.1) ; `Type::ExternApp`
(1.1).
**Produite** : dans la boucle d'`impl`, quand le `Self` d'un impl est une monomorphisation ayant un alias,
les ctors/méthodes sont émis sur le **type nominal de l'alias** (`extern type Vector3 { ctor new(...) = "nalgebra::Vector3::<f64>::new" }`).

## Étapes
1. Dans `impls.rs`, au traitement d'un bloc `impl` : résoudre le `Self`. Si `alias_for_monomorphization(Self)`
   renvoie un alias `A`, **router** les ctors/méthodes de ce bloc vers le type nominal `A` (et non vers la base).
2. Émettre le chemin Rust qualifié par l'alias (`crate::A::<args>::ctor`).
3. Si pas d'alias pour une monomorphisation concrète **et** que le ctor dépend d'une dimension non exprimable →
   `SkipReason::ConstGenericShape` (toujours pas de drop silencieux).

## Definition of Done (vérifiable)
- **Run réel** : `Vector3.new(1.0, 2.0, 3.0).norm()` (nalgebra) compile + produit le bon résultat.
- Le rapport de couverture montre la classe `ConstGenericShape` **en baisse** pour les ctors-dimension de
  nalgebra/ndarray.
- e2e du binding toujours VERT (corpus V12/V13 non régressé) ; 0 `unsafe`.

## Notes
- **Ne touche QUE `impls.rs`** → parallèle strict avec 4.2 (qui possède `mapty.rs`/`lower.rs`/`iri.rs`).
- Si l'alias porte des paramètres (`Vector3<T>`), réutiliser la mécanique de type-params d'impl déjà en place.
- Familles visées : nalgebra, ndarray, rustfft, rapier2d, smartcore (le **haut levier** de la cible 60/80).
