# Iron — V2 / `generated.md`

> Carte **provisoire** (🔶). Détail au **début du cycle V2**. Parent : `specs/generated.md`. Contrat : `generator.md`.

## Objectif

**Interop Rust par niveaux (I7)**, dans l'ordre imposé — ne pas spécifier un niveau avant le précédent.

## Specs

| ID | Composant | Niveau I7 | Invariants |
|---|---|---|---|
| 50100 | Blocs `rust: { }` inline | 1 | I7 |
| 50200 | Bindings `.iri` manuels | 2 | I7, I6 |
| 50300 | Auto-génération via `rustdoc --output-format json` | 3 | I7, I6 |

## Gate

Appel de crates Rust **réelles** depuis Iron aux 3 niveaux ; ordre I7 respecté ; **50300 (le plus dur) en fin de V2**.

## Pourquoi cette place

L'interop (dont `rust:` inline) sert d'**échappatoire** : en disposer **avant** les types avancés (V3) permet de contourner ce qui manque encore côté langage.
