# 33 — Barres de progression & spinners

> **Domaine** TUI terminal · **Crate(s)** `ironc add indicatif` · **Risque Iron** 🟢 · **Difficulté** modérée

## But
Habiller une tâche longue simulée (traitement de N éléments, boucle qui dort) d'une **barre de
progression** avec pourcentage, ETA et débit, plus une variante **spinner** (durée inconnue) et une
variante **multi-barres** pour plusieurs sous-tâches concurrentes.

## Ce que ça teste dans Iron
`indicatif` a une API **impérative et simple** : `ProgressBar::new(len)`, `.inc(1)`, `.set_message(...)`,
`.finish()`. On vérifie qu'Iron sait :
- appeler en boucle des **méthodes qui mutent l'état interne** d'un objet (le compteur), cet état étant
  encapsulé (`ProgressBar` est en réalité `Arc`-backed, *thread-safe*) ;
- styler la barre via un **template chaîne** (`ProgressStyle::with_template("{bar} {eta}")`) ;
- piloter un **`MultiProgress`** depuis plusieurs **threads Iron**. Aucune boucle d'événements clavier :
  pas le mur TUI, juste de l'affichage progressif.

## Approche
Créer une `ProgressBar` de longueur N, boucler le travail en appelant `.inc(1)` + `.set_message`, puis
`.finish_with_message`. Variantes : un spinner (`new_spinner`), et un `MultiProgress` alimenté par 2–3
threads qui avancent chacun leur barre.

## Risque / limites attendues
🟢 globalement OK. Réserve principale : `ProgressStyle::with_template` accepte parfois des **closures**
de formatage (`with_key`) → s'en tenir aux **templates chaîne** suffit ; éviter les closures custom
(frontière binder). La mutation du compteur est cachée derrière des méthodes simples → compatible avec
« méthode qui mute l'objet ». `MultiProgress` + threads peut exposer le besoin d'`Arc`/clone : Iron
clonant par valeur, surveiller le warning perf (I4). Attendu : passe, sauf insistance sur les closures.

## Done
Une barre qui progresse de 0 à 100 % avec ETA visible sur une tâche simulée ; bonus spinner + multi-
barres pilotées par threads. 0 `unsafe`. Noter si `with_template` (chaîne) a suffi ou si une closure
de style a été nécessaire.
