# 34 — Lecteur de logs temps réel (tail -f coloré)

> **Domaine** TUI terminal · **Crate(s)** `ironc add crossterm` · **Risque Iron** 🟡 · **Difficulté** modérée

## But
Suivre un fichier de log qui grossit (façon `tail -f`), afficher les nouvelles lignes au fil de l'eau,
**colorées par niveau** (ERROR rouge, WARN jaune, INFO vert), avec filtre optionnel par mot-clé.

## Ce que ça teste dans Iron
L'usage **le plus simple** de `crossterm` : pas de *raw mode* ni d'event loop clavier, seulement la
**colorisation** de la sortie (`SetForegroundColor`, `Print`, `ResetColor`). Le vrai cœur Iron testé est
ailleurs :
- **lecture incrémentale d'un fichier** en `std` : `seek(End)`, boucle « lire les octets ajoutés depuis
  la position courante → `sleep` → recommencer » ;
- une **boucle infinie à effet de bord I/O** sans boucle d'événements clavier — bien plus douce que les
  fiches 31/32 ;
- la classification de chaque ligne (sous-chaîne ou `regex`) pour choisir la couleur.

## Approche
Ouvrir le fichier, `seek` en fin ; boucle : lire les nouvelles lignes, classer le niveau, émettre la
ligne stylée, dormir ~200 ms. Quitter sur Ctrl-C. Filtre `--grep <mot>` optionnel qui saute les lignes
non concernées.

## Risque / limites attendues
🟡 modéré. Crossterm en mode « écriture stylée » est bien plus simple que son mode raw/event, **mais**
`execute!`/`queue!` sont des **macros** que le binder auto **ne voit pas** (macros ≠ fonctions) → appeler
les méthodes sous-jacentes (`queue`/`execute`) ou, plus sûr, **émettre les codes ANSI à la main** en
`std` (totalement faisable). La boucle `tail` (seek + sleep) est triviale en Iron. Piège à éviter : lire
le clavier pour filtrer en direct → on retomberait dans le mur event-loop. Attendu : passe si l'on évite
les macros (ANSI manuel) et la lecture clavier.

## Done
Suit un fichier qui grossit, affiche les nouvelles lignes colorées par niveau en temps réel ; filtre par
mot-clé optionnel ; sortie propre sur Ctrl-C. 0 `unsafe`. Noter explicitement : macros crossterm
contournées (ANSI/`std`) ou non.
