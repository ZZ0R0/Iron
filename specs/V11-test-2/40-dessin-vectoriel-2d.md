# 40 — Dessin vectoriel 2D (formes → PNG)

> **Domaine** Image & graphisme · **Crate(s)** `ironc add tiny-skia` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Composer une scène 2D — rectangles, cercles, chemins libres — avec remplissages et contours (et un
dégradé simple en bonus), sur un `Pixmap`, puis exporter en PNG.

## Ce que ça teste dans Iron
`tiny-skia` est un moteur de rendu 2D *immediate mode* **mais sans boucle d'événements** : `Pixmap::new`
(buffer mutable), `PathBuilder` (construire un chemin), `Paint`/`Stroke` (structs de style),
`pixmap.fill_path(&path, &paint, ...)`, `pixmap.save_png(...)`. On teste :
- les **builders** (`PathBuilder.move_to/line_to/finish`) et les **structs de configuration** ;
- surtout la **mutation en place d'un gros buffer** (`Pixmap`) à chaque opération de dessin, face à la
  sémantique par valeur ;
- le retour `Option`/`Result` de plusieurs constructions.

## Approche
Créer un `Pixmap`, un `Paint` (couleur, anti-alias) ; construire des `Path` via `PathBuilder` (rect,
cercle via `push_circle`, chemin libre) ; appeler `fill_path`/`stroke_path` sur le pixmap pour chaque
forme ; `save_png`. Un `LinearGradient` si le binder suit, sinon couleur unie.

## Risque / limites attendues
🟡. Points durs : (1) `PathBuilder` est un **builder consommant** (`finish(self) -> Option<Path>`) →
sémantique de *move* + retour `Option` à mapper ; (2) `Pixmap` est **muté en place** par chaque
`fill_path` → bon test du « méthode qui mute `self` » : un clone par opération serait catastrophique
(I4) ; (3) `Paint`/`Stroke`/`Transform` = structs à champs publics, devraient passer ; (4)
`LinearGradient::new(...)` prend un `Vec<GradientStop>` + enum `SpreadMode` + `Transform` → plus lourd,
optionnel. Pas d'`unsafe`, pas d'event loop → faisable, `.iri` léger probable pour `finish` et le
`Pixmap` mutable. Attendu : passe pour formes pleines/contours unis ; dégradé = bonus à risque.

## Done
Un PNG montrant au moins 3 primitives (rect, cercle, chemin) remplies et/ou contourées, couleurs
correctes. 0 `unsafe`. Noter le comportement du `Pixmap` mutable (clone ou non, perf `--audit`) et si
`PathBuilder.finish`/les dégradés ont exigé un `.iri`.
