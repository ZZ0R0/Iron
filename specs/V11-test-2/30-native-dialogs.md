# 30 — Dialogues natifs (ouvrir fichier) + notifications système
> **Domaine** GUI desktop · **Crate(s)** `ironc add rfd` + `ironc add notify-rust` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Ouvrir un **dialogue natif** « ouvrir fichier », lire le fichier choisi, puis lever une **notification système** indiquant ce qui a été ouvert. Pas de fenêtre applicative à nous : on appelle des dialogues **ponctuels** qui *rendent une valeur*.

## Ce que ça teste dans Iron
- Des **builders** simples : `FileDialog::new().add_filter(…).set_directory(…).pick_file()` et `Notification::new().summary(…).body(…).show()` — chaînage de constructeurs, le stress 🟡 classique du binder.
- Des **retours-valeurs** bien dans le modèle Iron : `pick_file` rend `Option<PathBuf>` (→ `match`/`Option` natif d'Iron), `show` rend un `Result` (→ I11). Aucune closure, aucun trait applicatif, **aucune boucle d'événements à posséder** : c'est ce qui le **sépare** des fiches 26-29.
- Interop `PathBuf` + lecture de fichier via `import std`.

## Approche
`FileDialog::new()…pick_file()` → `match` l'`Option` ; sur `Some`, lire le contenu (`import std`) ; déclencher une `Notification` portant le nom du fichier. Utiliser les API **synchrones** de rfd (éviter la variante async).

## Risque / limites attendues
🟡 : pas de boucle d'événements **possédée** ni de closure → on **évite le mur GUI**. Restent (1) les **builders** que le binder doit exposer (un petit `.iri` possible), (2) des **contraintes plateforme cachées** : le dialogue natif ouvre une fenêtre GTK/Cocoa et peut exiger le **thread principal** (notamment macOS) ; notify-rust passe par **D-Bus** sous Linux (un démon de notifications doit tourner). Ni headless-CI, ni *event loop* à gérer côté Iron → ça tient en 🟡.

## Done
Le dialogue s'ouvre, on choisit un fichier, son nom apparaît dans une notification système ; annuler le dialogue (`None`) ne crashe pas ; build vert sur une machine avec environnement graphique + démon de notifications.
