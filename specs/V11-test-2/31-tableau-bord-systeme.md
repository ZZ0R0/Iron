# 31 — Tableau de bord système (htop-like)

> **Domaine** TUI terminal · **Crate(s)** `ironc add ratatui sysinfo` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Un moniteur plein écran rafraîchi chaque seconde : jauges CPU et RAM, table des processus (PID, nom,
%CPU, mémoire) triable, sortie propre sur `q`. Le cas d'école du TUI « vivant ».

## Ce que ça teste dans Iron
La pire conjonction pour Iron, en trois couches :
- **`sysinfo`** expose un objet `System` qu'on **mute en place** (`refresh_cpu`, `refresh_memory`) à
  chaque frame — un état persistant qui traverse la boucle, à rebours de la sémantique par valeur.
- **`ratatui`** est de l'*immediate mode* piloté par une **boucle d'événements**, adossé à `crossterm`
  pour le *raw mode* et la lecture clavier non bloquante (`poll`/`read`).
- Les widgets se montent par **builders chaînés** (`Block::default().borders(...).title(...)`,
  `Gauge::default().ratio(...)`) et se rendent dans une **closure** (`terminal.draw(|f| ...)`).

## Approche
Entrer en *raw mode*, garder `System` + l'index sélectionné comme un objet vivant ; boucle : `poll`
clavier avec timeout → `refresh_*` → `terminal.draw(...)` qui peint les `Gauge` et la `Table`. À la
sortie, restaurer le terminal (`disable_raw_mode`, quitter l'écran alterné).

## Risque / limites attendues
🔴 triple mur. La **boucle d'événements + raw mode** est un effet de bord global persistant que
« valeur + pas d'état global mutable » modélise mal. `System` **doit** être muté sur place : un clone
par frame ruine sémantique et perf (I4). **Builders** ratatui et **closure de rendu** `terminal.draw`
sont hors de portée du binder auto → `.iri` manuel lourd. Piège final : sans `Drop` idiomatique, un
panic laisse le terminal cassé (raw mode non restauré). Issue probable : ne compile pas sans binding
manuel substantiel.

## Done
Soit (a) une carte précise des points de rupture (builders, closure `draw`, `System` mutable) avec la
sortie d'erreur réelle ; soit (b) version minimale via `.iri` manuel : jauges CPU+RAM rafraîchies,
sortie propre sur `q`, terminal restauré, 0 `unsafe`. Le livrable attendu est surtout **la carte du mur**.
