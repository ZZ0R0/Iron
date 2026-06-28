# 27 — Éditeur Markdown avec aperçu live
> **Domaine** GUI desktop · **Crate(s)** `ironc add egui` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Deux panneaux : à gauche une zone d'édition Markdown, à droite le **rendu live** qui se met à jour à chaque frappe. Le parsing Markdown est faisable en Iron ; la **coquille temps réel** est le mur.

## Ce que ça teste dans Iron
Tout le trio egui de la fiche 26 (boucle d'événements winit + `impl App` à `&mut self` persistant + closures `|ui| …` capturant l'état), **plus** deux durcissements :
- **Aliasing mutable dans un widget** : `TextEdit::multiline(&mut self.buffer)` confie une **référence mutable de ton état** au widget, qui **édite le buffer en place** à chaque frappe. C'est le cas I3 par excellence : auto-cloner ferait **perdre les frappes** (les éditions partiraient dans une copie jetée) → il faut un partage en place explicite, jamais un clone.
- **Re-parsing réactif** : à chaque `update`, on relit le buffer et on régénère le rendu — recomputation pilotée par la boucle.
- Le parseur (`pulldown-cmark`) émet un **flux d'events lazy** → friction binder côté lazy/itérateur.

## Approche (honnête)
Coquille à deux panneaux (`SidePanel`/colonnes, encore des closures) en `.iri`/`rust { }` ; le **mapping events Markdown → widgets egui** ; la liaison `&mut buffer`. Le parsing Markdown lui-même (itérer les events, bâtir une structure) est l'une des **rares parties réellement écrivables en Iron**.

## Risque / limites attendues
🔴 : même boucle d'événements que 26, aggravée par l'**emprunt mutable du buffer traversant une closure de widget** (interdit de cloner sous peine de perdre l'entrée utilisateur) — exactement la sémantique que « par valeur + clone-fallback » gère mal. Atterrissage : coquille en interop manuelle ; seul le parsing remonte côté Iron.

## Done
Taper `# Titre` à gauche affiche un titre rendu à droite **sans relancer** l'app ; aucune frappe perdue ; build vert. Documenter la part tombée en `rust { }`.
