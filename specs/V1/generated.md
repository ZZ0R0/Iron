# Iron — V1 / `generated.md`

> Carte **provisoire** (🔶) de V1. Détail complet au **début du cycle V1**, corrections de V0 intégrées.
> Parent : `specs/generated.md`. Contrat : `generator.md`.

## Objectif

**Qualité du Rust généré** : émettre des **emprunts** (`&`/`&mut`) là où c'est prouvé sûr, au lieu du clone-fallback systématique de V0. **Perf, pas correction** (I1 : le clone compile toujours).

## Specs

| ID | Composant | Dépend de | Invariants |
|---|---|---|---|
| 30200 | Élision d'emprunt (escape analysis) + warning clone-fallback | 30100, 40100 (V0) | I1, I3, I4 |

## Gate

Emprunts émis là où prouvé sûr ; `--audit` liste les clone-fallbacks (warning *perf*, off par défaut, I4) ; **sémantique strictement inchangée** vs V0 ; **aliasing mutable jamais auto-cloné** (I3 → exige `inout`/`ref` explicite) ; zéro `unsafe`.

## Pourquoi différé

V0 est déjà **correct** par clone-fallback (I1). L'élision est un gain de **perf** → on stabilise le cœur d'abord.
