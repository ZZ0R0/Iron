# 38 — Graphiques/courbes exportés en PNG

> **Domaine** Image & graphisme · **Crate(s)** `ironc add plotters` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Tracer une ou deux courbes (fonction mathématique, série de données) sur un canvas avec **axes, grille,
titre et légende**, puis exporter en PNG.

## Ce que ça teste dans Iron
`plotters` est l'archétype du crate **builder + closures + génériques**, mais sans boucle d'événements :
`BitMapBackend` → `into_drawing_area()` → `ChartBuilder::on(&area).caption(...).build_cartesian_2d(rx, ry)?`
→ `chart.draw_series(LineSeries::new(points, &RED))?`. On teste donc :
- le **chaînage profond** de builders et le `?` sur des `Result` de dessin partout ;
- les **types de coordonnées génériques** (`build_cartesian_2d`) ;
- les **closures** (`|x| (x, f(x))`) que les séries veulent — et la capacité d'Iron à les **éviter** ;
- les **références à des couleurs const** du crate (`&RED`, `&BLUE`).

## Approche
Backend bitmap 640×480 → drawing area → `ChartBuilder` (titre, plages, `configure_mesh`), puis
`draw_series(LineSeries::new(points, &RED))` où `points` est une **`Vec<(f64,f64)>` pré-calculée côté
Iron** (pas de closure passée au crate). Légende via `configure_series_labels`, puis `present()` → PNG.

## Risque / limites attendues
🟡 (voire 🔴 selon le binder). Murs concrets : (1) `build_cartesian_2d` **générique** sur les types
d'axes ; (2) `LineSeries::new` veut un **itérateur de tuples**, d'ordinaire issu d'une closure/`map` →
**contournement clé : pré-calculer la `Vec<(f64,f64)>`** en Iron ; (3) `&RED`/`&BLUE` = **références à
des consts** du crate, mapping à vérifier ; (4) `?` omniprésent. Un `.iri` manuel sera probablement
nécessaire pour discipliner les génériques. Pas d'état global ni d'event loop → faisable, mais
l'ergonomie pique. Attendu : passe au prix d'un `.iri` et en pré-calculant les séries.

## Done
Un PNG avec au moins une courbe lisible, axes gradués, titre et légende. 0 `unsafe`. Documenter quelles
parties ont exigé un binding manuel (builder `ChartBuilder`, `LineSeries`, couleurs const) et confirmer
que **pré-calculer les points** évite de passer des closures au crate.
