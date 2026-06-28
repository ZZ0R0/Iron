# 69 — Lecteur de traces GPS (parse GPX, stats)
> **Domaine** Géo, temps & docs · **Crate(s)** `ironc add gpx` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Parser un fichier **GPX** (trace GPS) et calculer des statistiques : distance totale, dénivelé positif, durée, vitesse moyenne.

## Ce que ça teste dans Iron
- `gpx::read(reader)` = **API « feuille »** rendant une struct `Gpx` à nœuds **imbriqués** (tracks → segments → points).
- Beaucoup de **champs `Option`** (élévation, horodatage parfois absents) → gymnastique `Option` d'Iron.
- Itération sur un `Vec` de points + maths haversine ; le champ `time` est un type `chrono`/`time` (interop croisée).

## Approche
Lire via un `BufReader`, parcourir tracks/segments/points, sommer les distances haversine entre points consécutifs, accumuler les deltas d'altitude positifs, déduire la durée des horodatages et la vitesse moyenne. Gérer proprement les points sans altitude/temps.

## Risque / limites attendues
- 🟢 Le **parse** est simple et rend des structs ordinaires ; la friction est surtout les **champs optionnels profondément imbriqués** (`Option` partout) à déballer sans paniquer.
- La distance impose la **formule haversine** (manuelle ou via `geo`, cf. 67) — petite dépendance logique.
- Champ temporel = type tiers (`chrono`/`time`) : une 2ᵉ dépendance implicite à exposer.

## Done
- Parse d'un GPX réel ; distance/dénivelé/durée **plausibles** vs un outil de référence.
- Points sans champ optionnel gérés en `Option`, **zéro panic**.
- Vitesse moyenne cohérente (distance/durée).
