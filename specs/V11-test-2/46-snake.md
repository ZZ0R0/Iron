# 46 — Jeu Snake
> **Domaine** Jeux · **Crate(s)** `ironc add macroquad` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Un Snake jouable : fenêtre graphique, serpent qui avance aux flèches, grandit en mangeant, meurt en se mordant ou en heurtant un mur ; score affiché.

## Ce que ça teste dans Iron
- **Trois murs d'un coup.** (1) `#[macroquad::main("Snake")]` est un **attribut proc-macro** qui génère un point d'entrée **async** — ni les macros d'attribut ni `async`/`await` n'ont de surface dans le transpileur Iron.
- (2) La **boucle de jeu** est un `loop { …; next_frame().await }` infini : boucle d'événements + état mutable par frame (corps du serpent, direction, pomme, score) muté à chaque tour → collision frontale avec « par valeur, pas d'état global mutable ».
- (3) macroquad s'appuie sur un **contexte graphique global** caché (`draw_rectangle`, `is_key_down` lisent/écrivent un état thread-local) — précisément l'état global mutable qu'Iron bannit côté utilisateur.
- Le **timing par frame** (`get_frame_time` + accumulateur pour avancer le serpent à intervalle fixe) dépend entièrement de la boucle async — indisponible tant que l'entrée ne passe pas.

## Approche
`ironc add macroquad`. *Conceptuellement* : initialiser l'état, puis chaque frame lire l'entrée, avancer la tête sur la grille, gérer collision/croissance, redessiner. La logique de grille (déplacement, détection de collision) est **pure et par valeur** : c'est la partie qu'Iron sait faire. Le reste est la coquille macroquad. Repli pour au moins tester le jeu : extraire le pas de jeu en fonction pure et le piloter depuis un harnais texte, sans macroquad.

## Risque / limites attendues
- L'entrée **async par attribut** (`#[macroquad::main]` + `next_frame().await`) est probablement **inexprimable** telle quelle : c'est le blocage dur.
- L'état muté chaque frame doit, en sémantique par valeur, être **reconstruit** à chaque tour (style fonctionnel) — faisable pour la *logique*, mais la *boucle* elle-même bute sur l'async.
- Cartographie attendue : confirmer que « entrée async + boucle d'événements » est une arête dure d'Iron, et noter exactement où l'on cale.
- Même en contournant l'attribut, `next_frame().await` impose un **exécuteur async** qu'Iron ne fournit pas — le mur reste.

## Done
*Visé* : fenêtre, serpent qui bouge, grandit, meurt ; score. *Réaliste* : arrêt attendu à la frontière `main` async / `next_frame().await` ; documenter le point de blocage.
