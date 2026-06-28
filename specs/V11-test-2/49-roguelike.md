# 49 — Roguelike (grille texte, déplacements)
> **Domaine** Jeux · **Crate(s)** `ironc add bracket-lib` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Un mini-roguelike : carte en grille de texte, un `@` qui se déplace aux flèches/touches vi, murs qui bloquent, idéalement un champ de vision (FOV).

## Ce que ça teste dans Iron
- **Le mur du contrôle inversé.** bracket-lib impose d'implémenter le trait **`GameState`** sur ton état, avec `fn tick(&mut self, ctx: &mut BTerm)`, puis de confier l'état à `main_loop`, qui **possède la boucle** et te **rappelle** à chaque frame.
- Cette **inversion** (le framework t'appelle) exige un **objet d'état à longue vie, muté en place** (`&mut self`) à chaque frame → collision directe avec « par valeur, pas d'état global mutable ».
- `main_loop` **prend possession** du contexte + de l'état et **ne revient jamais** (diverge) en pilotant sa propre boucle.
- `BTermBuilder` est un **builder chaîné** (`simple80x50().with_title().build()`), motif que le binder digère mal sans `.iri` — double peine avec l'inversion de contrôle.

## Approche
`ironc add bracket-lib`. *Conceptuellement* : un état (carte, position du joueur), `tick` qui lit l'entrée, déplace `@` si la case est libre, redessine. La **génération de carte, le FOV et l'arithmétique de grille** sont purs et par valeur — la partie qu'Iron sait faire ; les isoler en fonctions testées via un harnais texte, sans bracket-lib. Le harnais `GameState` + `main_loop` est ce qui révèle la limite.

## Risque / limites attendues
- Implémenter un **trait étranger** (`GameState`) dont la méthode prend `&mut self` **et** un `&mut BTerm`, puis passer l'objet à une boucle possédée par le framework, est exactement l'**arête du contrôle inversé** : blocage dur probable.
- `main_loop` qui **diverge** (ne retourne jamais) cadre mal avec un `main` par valeur classique.
- Le cœur (carte/FOV/déplacement) transpile bien isolé ; documenter que c'est **l'inversion de contrôle** qui cale, pas la logique.
- bracket-lib vise aussi le **wasm** et embarque son propre exécuteur : l'intégration suppose un point d'entrée non standard, étranger au `main` d'Iron.

## Done
*Visé* : carte affichée, `@` qui bouge, murs bloquants, FOV. *Réaliste* : arrêt à l'inversion `GameState`/`main_loop` ; coder et valider le cœur logique à part, et noter précisément le point de blocage.
