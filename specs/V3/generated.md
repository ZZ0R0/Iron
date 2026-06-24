# Iron — V3 / `generated.md`

> Carte **provisoire** (🔶). Détail au **début du cycle V3**. Parent : `specs/generated.md`. Contrat : `generator.md`.

## Objectif

**Système de types avancé** — le gros morceau dur : `enum` + `match` + traits + génériques, en **projection propre d'un sous-ensemble Rust (I6)**.

## Specs

| ID | Composant | Dépend de | Invariants |
|---|---|---|---|
| 20300 | Enums + match + traits + génériques | inférence V0 (20100/20200) | I6 |

## Gate

Lowering **idiomatique** vers Rust ; projection I6 tient ; **toute construction non mappable vers Rust justifiée** dans la spec (I6).

## Pourquoi repoussé

Le plus dur conceptuellement ; isolé en fin de plan pour **ne pas bloquer** le cœur e2e (V0) ni l'interop (V2).
