# 29 — Sélecteur de couleurs (sliders RGB/HSV)
> **Domaine** GUI desktop · **Crate(s)** `ironc add egui` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Des sliders **R/G/B** et **H/S/V** liés : bouger l'un recalcule l'autre espace et met à jour un aperçu de couleur. La **conversion RGB↔HSV** est du pur Iron (bon candidat à écrire réellement) ; la coquille egui reste le mur.

## Ce que ça teste dans Iron
Le trio egui des fiches 26/27 (boucle winit + `impl App` `&mut self` + closures), **spécialisé sur l'aliasing répété** :
- **N sliders, N emprunts mutables** : chaque `Slider::new(&mut self.r, 0..=255)` confie **un `&mut` d'un champ** au widget — l'aliasing mutable I3 répété six fois, qui doit **partager en place** (cloner perdrait le déplacement du slider).
- **Cohérence d'état réactive** : bouger R doit recalculer H/S/V **dans le même `&mut self`** et se refléter sur les autres sliders au frame suivant — propagation d'une mutation vers des champs dérivés, en sémantique par valeur.
- Implémentation du **trait `App`** + closures `|ui| …` (comme 26).

## Approche (honnête)
La conversion RGB↔HSV s'écrit **proprement en Iron** (classe couleur + méthodes + surcharge d'opérateurs éventuelle) et constitue l'unité réellement testable. La coquille (sliders liés, `impl App`) passe par `.iri`/`rust { }`. On isole « logique Iron » / « coquille interop ».

## Risque / limites attendues
🔴 : c'est le 🔴 « le plus léger » (logique triviale), mais **le mur de la coquille est identique** aux autres egui : boucle d'événements + **multiples `&mut` de champs traversant des widgets** (I3, interdit de cloner) + état réactif possédé par le framework. Faisable **seulement** avec coquille en interop manuelle ; la conversion couleur, elle, valide bien Iron.

## Done
Bouger le slider R change l'aperçu **et** les valeurs HSV de façon cohérente (et inversement) ; build vert. Conversion en Iron, coquille en `rust { }` si nécessaire — **documenté**.
