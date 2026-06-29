# 26 — Plus court chemin dans un graphe (Dijkstra)
> **Domaine** Algorithmes/graphes · **Crate(s)** `ironc add petgraph` · **Risque Iron** 🟡 · **Difficulté** élevée

## But
Construire un graphe pondéré, ajouter des nœuds et des arêtes, puis calculer le plus court chemin entre
deux nœuds (`dijkstra`) et afficher le coût total.

## Ce que ça teste dans Iron
Un crate **fortement générique** : `Graph<N, E, Ty, Ix>` (poids de nœud, d'arête, orientation, type
d'index). Les `NodeIndex`/`EdgeIndex` sont des handles. Teste la projection de génériques multi-paramètres
et les algos libres prenant `&Graph` + une closure de coût.

## Approche
`Graph.new()` → `add_node(w)` → `add_edge(a, b, poids)` → `dijkstra(&g, source, None, |e| *e.weight())`.
Cible : un petit graphe à 4-5 nœuds dont le plus court chemin est connu.

## Done
`ironc run` imprime le coût du plus court chemin (valeur attendue connue), build sans `unsafe`.
