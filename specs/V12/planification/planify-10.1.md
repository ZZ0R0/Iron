# planify-10.1 — Mode d'émission **`#![no_std]`** + HAL béni

- Réalise (cadrage) : [43000], [44000]
- Version : V12 · Vague : 10 · Piste : 1 · Statut : todo
- Dépend de : planify-8.1 (émission), planify-9.1 (un driver embarqué = souvent un trait HAL implémenté)

## But
Émettre un crate **`#![no_std]`** + `extern crate alloc` ciblant une **crate-HAL bénie** (esp-hal, rppal,
linux-embedded-hal), **sans qu'Iron émette d'`unsafe`** — l'`unsafe` reste **dans le HAL** (I2 intact).
Correction d'un cadrage erroné : ce n'est **pas** une violation d'invariant, c'est un **mode d'émission**.

## Fichiers POSSÉDÉS (ownership disjoint)
- **Édition** : `ironc/src/lower.rs` (directive `#![no_std]` + `alloc`), `ironc/src/driver.rs` (cible Cargo
  no_std : profil, dep HAL, pas de `std`).
- Séquentielle (vague 10).

## Interface produite / consommée
- **Consomme** : couche d'émission (8.1), bindings HAL (via le pont + impl-trait 9.1).
- **Produit** : crate généré `no_std` qui cross-compile contre un HAL.

## Étapes
1. Mode projet `no_std` (option `ironc.toml`/directive) : émettre `#![no_std]` + `extern crate alloc` ; mapper
   `String`/`Vec` Iron sur `alloc::*`.
2. driver : profil de build/cible (ex. `thumbv*`/`xtensa`), dépendance HAL déclarée.
3. Vérifier **0 `unsafe` émis** par Iron (l'`unsafe` est dans le HAL compilé).

## Definition of Done (vérifiable)
1. Un programme Iron minimal (blink LED via HAL) **cross-compile** en `no_std` ; `grep` : 0 `unsafe` dans le Rust émis (I2).
2. **Gate VERT** (le mode std reste le défaut, inchangé).

## Notes
- Le *run* exige le **matériel** (environnement, hors langage). On valide la **cross-compilation**, pas le flash.
- Liste des HAL bénis = décision de cadrage (sécurité I2) — un HAL n'est admis que s'il encapsule son `unsafe`.
