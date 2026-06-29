# 74 — Markdown → HTML (parseur à événements)
> **Domaine** Parsing/documents · **Crate(s)** `ironc add pulldown-cmark` · **Risque Iron** 🟥 · **Difficulté** élevée

## But
Parser du Markdown et le rendre en HTML, ou compter les éléments (titres, liens) en parcourant le flux
d'événements.

## Ce que ça teste dans Iron
`pulldown-cmark` est un **parseur à itérateur d'événements** (`Parser<'a>` rend des `Event<'a>` empruntant
l'entrée) — durées de vie + énum riche d'événements. Proche de quick-xml (io par flux/événements, [52800]).

## Approche
`Parser.new(md)` → `push_html(&mut out, parser)` (rendu direct) si projeté ; sinon compter les événements.
Mesurer la couverture si l'itérateur d'événements ne se projette pas.

## Done
`ironc run` rend un `# Titre` en `<h1>Titre</h1>`, ou couverture mesurée + rapport si muré.
