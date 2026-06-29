# 77 — Conversions d'espaces colorimétriques
> **Domaine** Couleur/graphisme · **Crate(s)** `ironc add palette` · **Risque Iron** 🟥 · **Difficulté** très élevée

## But
Convertir une couleur RGB → HSL → Lab, mélanger deux couleurs, ajuster la luminosité.

## Ce que ça teste dans Iron
`palette` est de la **programmation au niveau type** sur les espaces de couleur : `Srgb<T>`, `Hsl<S, T>`,
`Lab<Wp, T>` (point blanc en paramètre fantôme). Un des crates les plus durs (type-level + génériques de
marqueur). Test limite du moteur, voisin de uom.

## Approche
Mesurer d'abord la **couverture** de `ironc add palette` + rapport. Si tractable, cibler `Srgb<f32>` →
`Hsl` via `from_color` ; sinon documenter le mur précis.

## Done
Au minimum : couverture mesurée + rapport des sautés. Idéalement : une conversion RGB→HSL imprimée.
