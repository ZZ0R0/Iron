# 71 — Blink LED sur ESP32 (firmware no_std)
> **Domaine** Embarqué & matériel · **Crate(s)** `ironc add esp-hal` · **Risque Iron** 🔴🔴 · **Difficulté** modérée

## But
Faire clignoter une LED sur un microcontrôleur ESP32 — le « hello world » du bare-metal : configurer une
broche en sortie et l'alterner haut/bas dans une boucle avec un délai. Scope minuscule, mais c'est le
projet qui **touche le mur le plus profond** d'Iron : il n'est pas « modéré », il est révélateur.

## Ce que ça teste dans Iron
Il viole **quatre** garanties d'Iron **simultanément** :
- **`no_std`** : un firmware n'a ni OS ni libc, donc pas de `std`. Or Iron **émet toujours du `std`**
  (`String`, `Vec`, allocateur, `println!`). Conflit frontal, non contournable sans nouveau mode.
- **`unsafe`** : piloter une broche = écrire un registre mappé en mémoire → `unsafe` obligatoire,
  qu'Iron **n'émet jamais** (I2).
- **Cross-compilation** : cible `xtensa-esp32-none-elf` (LLVM forké, hors Rust mainline) ou
  `riscv32imc-*` pour les ESP32-C ; Iron compile pour l'hôte via `cargo`.
- **État global mutable** : `Peripherals`, timer, GPIO sont des singletons partagés (statics, ISR) —
  interdits par « pas d'état global mutable ».

## Approche
Aucune voie « tranquille » n'existe. Au mieux on **constate** : `esp-hal` est un HAL saturé de type-states
(`GpioPin<Output, N>`), de const-génériques et de la macro `#[entry]` (pas de `main` std) — le binder auto
V8 ne sait pas l'exprimer. Un vrai blink supposerait un **mode `no_std` dédié** dans Iron (chantier de
cadrage séparé), une **dérogation `unsafe` scellée**, le plumbing de cross-cible et un modèle d'ownership
des périphériques. C'est un projet en soi, pas une fiche d'application.

## Risque / limites attendues
🔴🔴 — **probablement infaisable en l'état**, et c'est le **résultat attendu**, pas un échec. La fiche
existe pour matérialiser la frontière : l'embarqué bare-metal est hors du domaine d'Iron tant que
`no_std` + `unsafe` ne sont pas traités au cadrage. C'est la sonde la plus profonde du lot.

## Done
Le livrable **honnête** n'est pas une LED qui clignote, mais un **constat documenté** : les 4 conflits
nommés, et la liste minimale d'extensions Iron (mode `no_std`, dérogation unsafe encapsulée, cibles
xtensa/riscv) requises avant d'espérer compiler ne serait-ce qu'un squelette qui linke.
