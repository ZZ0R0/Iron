# 48 — Démineur (en terminal)
> **Domaine** Jeux · **Crate(s)** `ironc add crossterm` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Un démineur jouable au terminal : grille de cases, curseur déplaçable aux flèches, révélation/drapeau, propagation (flood-fill) sur les cases vides, détection de victoire/défaite, terminal restauré à la sortie.

## Ce que ça teste dans Iron
- crossterm est une **bibliothèque, pas un runtime** : c'est **toi** qui écris la boucle (`loop { dessiner(); match event::read() { … } }`) en Iron pur — pas d'async, `event::read()` est bloquant. D'où 🟡 et non 🔴.
- Le frottement : `execute!`/`queue!` sont des **macros variadiques** (pipelines de commandes) ; Iron n'a pas de surface macro → il faut appeler les **méthodes de trait** sous-jacentes (`ExecutableCommand`/`QueueableCommand`) à la place.
- Le **raw-mode** est un effet de bord global du terminal (entrée/sortie) à **restaurer** proprement (RAII ou try/finally).
- L'état (grille de mines, masque révélé, drapeaux) muté à chaque entrée → reconstructible par valeur.
- Le placement aléatoire des mines (`rand`) **après** le premier clic (pour ne pas perdre au coup 1) : logique pure de génération de grille.

## Approche
`ironc add crossterm`. Activer le raw-mode, masquer le curseur, boucler : dessiner la grille, lire un événement clavier, mettre à jour l'état (déplacer le curseur, révéler/marquer), redessiner. Le **flood-fill** des cases vides est de la logique pure. Calculer une fois les compteurs de voisinage, puis ne redessiner que sur changement d'état pour limiter le scintillement. Restaurer le terminal en sortie quoi qu'il arrive.

## Risque / limites attendues
- Les **macros** `execute!`/`queue!` n'existent pas en Iron : les réécrire en appels de méthodes de trait sur `stdout` est le principal travail de binding.
- Le **cycle de vie du raw-mode** doit être garanti même en cas de panique, sinon le terminal reste cassé : tester la restauration.
- L'enum `KeyCode` à matcher et le `poll` optionnel sont mécaniques. Pas de mur dur attendu.
- Le rendu couleur des chiffres 1–8 repasse par des commandes crossterm (`SetForegroundColor`) — encore des appels de trait, pas de macro.

## Done
La grille s'affiche, les flèches déplacent le curseur, révélation/drapeau marchent, le flood-fill ouvre les zones vides, victoire/défaite détectées, et le terminal est rendu intact à la sortie.
