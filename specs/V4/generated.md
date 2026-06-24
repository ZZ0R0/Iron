# Iron — V4 / `generated.md`

> Carte **provisoire** (🔶). Détail au **début du cycle V4**. Parent : `specs/generated.md`. Contrat : `generator.md`.

## Objectif

**Stdlib** : wrappers **fins** sur `std` + crates (I7). Pas de runtime lourd (I5).

## Specs

| ID | Composant | Dépend de | Invariants |
|---|---|---|---|
| 80100 | Stdlib (wrappers fins sur std + crates) | interop V2, types V3 | I7 |

## Gate

Surface stdlib **utilisable** ; wrappers fins (pas de runtime lourd, I5) ; I7 respecté.

## Pourquoi dernier

Dépend de l'**interop (V2)** et potentiellement des **génériques (V3)** → vient en dernier.
