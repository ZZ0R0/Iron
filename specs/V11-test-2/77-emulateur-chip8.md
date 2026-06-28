# 77 — Émulateur CHIP-8
> **Domaine** Langages & exotiques · **Crate(s)** `ironc add minifb` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Émuler la machine virtuelle CHIP-8 : charger une ROM, exécuter ses opcodes, afficher le framebuffer 64×32
dans une fenêtre et lire le clavier 16 touches. Objectif concret : faire tourner un vrai jeu (PONG, etc.).

## Ce que ça teste dans Iron
- Le **cœur CPU** est de la logique pure très Iron-friendly : `[u8; 4096]` de RAM, 16 registres, pile,
  **décodage d'opcodes** par nibbles (`match` + masques de bits `>> & |`) ;
- **arithmétique wrappante** : CHIP-8 dépend de l'addition avec carry/borrow (`wrapping_add`, flag `VF`) —
  Iron doit exposer les ops wrappantes et un débordement **déterministe**, pas un panic ;
- des **tableaux de taille fixe** et l'indexation sûre ;
- `minifb` : une **boucle de rendu** (`Window::new`, `update_with_buffer(&[u32])`, `is_key_down`) — le motif
  boucle-d'événements qui frotte avec « pas d'état global mutable ».

## Approche
Séparer **core** (fetch/decode/execute + timers 60 Hz) et **frontend** minifb (blit du buffer monochrome
vers `&[u32]`, lecture clavier). Le core se teste sans fenêtre ; minifb ne fait que pousser un buffer et
sonder les touches — boucle légère, pas d'arbre de widgets retenu.

## Risque / limites attendues
🟡 — le core seul serait 🟢 ; ce sont minifb (boucle de rendu + FFI `&[u32]`) et l'**arithmétique
wrappante** qui montent le risque à 🟡. Bien plus simple qu'un vrai jeu (46/47) : pas d'event-loop retenu,
juste un blit par frame. Bon test du « jeu minimal » sans le poids d'un moteur.

## Done
Émulateur qui charge une ROM, exécute correctement le jeu d'opcodes (validé sur une ROM de test connue),
affiche l'écran dans la fenêtre minifb et répond au clavier.
