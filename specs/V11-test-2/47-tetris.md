# 47 — Jeu Tetris
> **Domaine** Jeux · **Crate(s)** `ironc add macroquad` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Un Tetris jouable : pièces qui tombent, rotation, déplacement latéral, verrouillage, effacement des lignes pleines, accélération de la gravité, game over en haut de puits.

## Ce que ça teste dans Iron
- **Le même mur moteur que Snake** : `#[macroquad::main]` async + `loop { …; next_frame().await }` + contexte de dessin global. L'entrée async et la boucle d'événements restent inexprimables telles quelles.
- Un **état mutable plus lourd** : la grille du puits (tableau 2D modifié en continu) + la pièce active (position, rotation). En sémantique par valeur, chaque tick **réalloue** une nouvelle grille au lieu de muter en place.
- Bon côté : la **rotation de pièce** et les **tests de collision** sont de la logique **pure** (matrices, coordonnées) — la surcharge d'opérateurs d'Iron sert bien à l'arithmétique de vecteurs 2D.
- Le **sac de pièces** (7-bag) et l'apparition aléatoire : `rand` + état de séquence, reconstruit par valeur à chaque pioche.

## Approche
`ironc add macroquad`. Séparer nettement le **cœur pur** (rotation, collision, effacement de lignes, scoring — tout par valeur, parfaitement transpilable) de la **coquille macroquad** (fenêtre, boucle, timing de gravité, polling clavier). Coder et tester le cœur d'abord ; la coquille est ce qui révèle la limite. Représenter chaque tétromino par ses offsets relatifs et tourner par rotation 2D — pur, testable hors macroquad.

## Risque / limites attendues
- Identique à Snake : l'**entrée async** et `next_frame().await` sont le blocage dur ; le **timing** de gravité (chrono par frame) dépend de la boucle, donc indisponible tant que la coquille ne passe pas.
- La logique pure transpile bien ; c'est l'**enveloppe événementielle** qui cogne. Distinguer les deux dans les notes de test.
- Comme Snake, l'`await` par frame exige un exécuteur async absent d'Iron : le cœur passe, la coquille non.

## Done
*Visé* : pièces qui tombent et tournent, lignes effacées, score, game over. *Réaliste* : le cœur logique passe en isolé ; arrêt à la même frontière async/boucle que Snake — le documenter.
