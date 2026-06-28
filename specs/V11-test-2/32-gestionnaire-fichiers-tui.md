# 32 — Gestionnaire de fichiers TUI

> **Domaine** TUI terminal · **Crate(s)** `ironc add ratatui crossterm` · **Risque Iron** 🔴 · **Difficulté** modérée

## But
Naviguer l'arborescence dans le terminal : liste du répertoire courant, flèches haut/bas pour la
sélection, Entrée pour descendre (ou ouvrir un fichier via `$EDITOR`/`xdg-open`), Backspace pour
remonter, ligne sélectionnée en surbrillance.

## Ce que ça teste dans Iron
Même cœur TUI que la fiche 31 (boucle d'événements `crossterm` + *raw mode* + builders `ratatui`),
augmenté de deux difficultés :
- un **état de navigation mutable riche** : `cwd: PathBuf`, `Vec` d'entrées trié (lu via
  `std::fs::read_dir`), `selected: usize`, pile de répertoires — muté à chaque touche.
- le **`ListState`** de ratatui, un état *stateful* passé à `render_stateful_widget`, plus le **lancement
  d'un process externe** (`std::process::Command`) qui impose de **suspendre puis restaurer** la TUI.

## Approche
Lister le répertoire en `Vec` trié (dossiers d'abord) ; boucle : lire l'événement clavier, muter
sélection/`cwd`, redessiner la `List` avec l'item courant surligné. Pour ouvrir : quitter le raw mode,
*spawn* l'éditeur, ré-entrer dans la TUI à son retour.

## Risque / limites attendues
🔴 mêmes murs que 31 (event loop, raw mode, builders, closure de rendu) plus le **`ListState`** (état
muté hors du modèle par valeur) et la **séquence sortir/relancer/rentrer** autour du process externe —
un enchaînement d'effets de bord que la sémantique par valeur ne capture pas. La pile de navigation
(`Vec<PathBuf>` mutée) et le mapping des `KeyCode` (enum) sont faisables, mais le couplage
boucle-clavier-rendu reste du binding manuel. Sans `.iri` substantiel : échec de binding probable.

## Done
Navigation haut/bas + descendre/remonter opérationnelle, sélection visible, ouverture d'un fichier puis
retour propre, terminal restauré, 0 `unsafe`. À défaut : rapport précis des constructions ratatui/
crossterm non bindables et du coût du `.iri` manuel.
